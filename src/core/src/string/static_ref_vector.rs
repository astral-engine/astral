// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	cell::UnsafeCell,
	hint, mem,
	num::NonZeroU32,
	sync::atomic::{self, AtomicUsize},
};

use super::{USED_MEMORY, USED_MEMORY_CHUNKS};

const ELEMENTS_PER_PAGE: usize = 64 * 1024 / mem::size_of::<usize>();

type Page<'a, T> = Box<[Option<&'a T>; ELEMENTS_PER_PAGE]>;

/// A vector which stores immutable pointers to `T`.
///
/// Retrieving the pointers is implemented wait-free. Pushing new pointers
/// however requires external synchronization.
pub struct StaticRefVector<'a, T> {
	pages: Box<[UnsafeCell<Option<Page<'a, T>>>]>,
	len: AtomicUsize,
}

impl<'a, T> StaticRefVector<'a, T>
where
	T: 'a,
{
	/// Constructs a new, empty vector with the specified capacity.
	///
	/// The capacity cannot be changed afterwards. Otherwise it would not be
	/// possible to access elements in a wait-free thread-safe manner.
	pub fn new(capacity: usize) -> Self {
		let needed_pages =
			(capacity + ELEMENTS_PER_PAGE - 1) / ELEMENTS_PER_PAGE;
		USED_MEMORY.fetch_add(
			mem::size_of::<UnsafeCell<Option<Page<'a, T>>>>() * needed_pages,
			atomic::Ordering::Acquire,
		);
		USED_MEMORY_CHUNKS.fetch_add(1, atomic::Ordering::Acquire);
		let mut pages = Vec::with_capacity(needed_pages);
		for _ in 0..needed_pages {
			pages.push(UnsafeCell::new(None));
		}
		Self {
			pages: pages.into_boxed_slice(),
			len: AtomicUsize::new(0),
		}
	}

	/// Returns the number of elements stored in the vector.
	pub fn len(&self) -> usize {
		self.len.load(atomic::Ordering::Acquire)
	}

	/// Returns `true` if the vector is empty.
	#[allow(unused)]
	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the number of used pages.
	pub fn num_pages(&self) -> usize {
		let len = self.len();
		if len == 0 {
			0
		} else {
			Self::page_index(len) + 1
		}
	}

	const fn page_index(index: usize) -> usize {
		index / ELEMENTS_PER_PAGE
	}

	const fn element_index(index: usize) -> usize {
		index % ELEMENTS_PER_PAGE
	}

	// Returning `mut` is allowed because of `UnsafeCell`
	#[allow(clippy::mut_from_ref)]
	unsafe fn page(&self, index: usize) -> &mut Option<Page<'a, T>> {
		let page_index = Self::page_index(index);
		debug_assert!(page_index < self.pages.len());
		&mut *self.pages.get_unchecked(page_index).get()
	}

	// Returning `mut` is allowed because of `UnsafeCell`
	#[allow(clippy::mut_from_ref)]
	unsafe fn page_or_create(&self, index: usize) -> &mut Page<'a, T> {
		let page = self.page(index);
		if let Some(page) = page {
			page
		} else {
			Option::replace(page, Box::new(mem::zeroed()));
			page.as_mut().unwrap_or_else(|| {
				debug_assert!(false, "page was not created");
				hint::unreachable_unchecked();
			})
		}
	}

	unsafe fn element<'page>(
		page: &'page mut Page<'a, T>,
		index: usize,
	) -> &'page mut Option<&'a T> {
		debug_assert!(index < ELEMENTS_PER_PAGE);
		page.get_unchecked_mut(index)
	}

	/// Appends an element to the back of the vector.
	///
	/// # Safety
	///
	/// This is unsafe because pushing to the collection is not thread safe.
	#[allow(clippy::cast_possible_truncation)]
	pub unsafe fn push(&self, value: &'a T) -> NonZeroU32 {
		let index = self.len.load(atomic::Ordering::Relaxed);

		let page = self.page_or_create(index);
		let element = Self::element(page, Self::element_index(index));
		debug_assert!(element.is_none());
		Option::replace(element, value);

		self.len.store(index + 1, atomic::Ordering::Release);
		NonZeroU32::new_unchecked(index as u32 + 1)
	}

	/// Returns the pointer at the given index, without doing bounds checking.
	pub unsafe fn get_unchecked(&self, index: NonZeroU32) -> &'a T {
		let index = index.get() as usize - 1;
		let page = self.page(index).as_mut().unwrap_or_else(|| {
			debug_assert!(false, "page was not created");
			hint::unreachable_unchecked();
		});
		Self::element(page, Self::element_index(index)).unwrap_or_else(|| {
			debug_assert!(false, "element does not exist");
			hint::unreachable_unchecked();
		})
	}

	/// Returns the pointer at the given index or [`None`] if the index is out of bound.
	pub fn get(&self, index: NonZeroU32) -> Option<&'a T> {
		if (index.get() as usize - 1) < self.len() {
			unsafe { Some(self.get_unchecked(index)) }
		} else {
			None
		}
	}
}

impl<'a, T> Drop for StaticRefVector<'a, T> {
	fn drop(&mut self) {
		let pages = self.num_pages();
		unsafe {
			for page in self.pages.iter_mut().take(pages) {
				(*page.get()).take();
			}
		}
	}
}

unsafe impl<'a, T> Send for StaticRefVector<'a, T> {}
unsafe impl<'a, T> Sync for StaticRefVector<'a, T> {}
