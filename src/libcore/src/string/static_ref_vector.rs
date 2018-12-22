// Copyright (C) Astral Developers - All Rights Reserved
// Unauthorized copying of this file, via any medium is strictly prohibited
// Proprietary and confidential
// Written by Tim Diekmann <tim.diekmann@3dvision.de>, November 2018

use std::{
	cell::UnsafeCell,
	hint,
	mem,
	sync::atomic::{self, AtomicUsize},
};

use super::StringId;

const ELEMENTS_PER_PAGE: usize = 64 * 1024 / mem::size_of::<usize>();

type Page<T> = Box<[Option<*const T>; ELEMENTS_PER_PAGE]>;

/// A vector which stores immutable pointers to `T`.
///
/// Retrieving the pointers is implemented wait-free. Pushing new pointers
/// however requires external synchronization.
pub(super) struct StaticRefVector<T> {
	pages: Box<[UnsafeCell<Option<Page<T>>>]>,
	len: AtomicUsize,
}

impl<T> StaticRefVector<T> {
	/// Constructs a new, empty vector with the specified capacity.
	///
	/// The capacity cannot be changed afterwards. Otherwise it would not be
	/// possible to access elements in a wait-free thread-safe manner.
	pub(super) fn new(capacity: usize) -> (Self, usize, usize) {
		let needed_pages = (capacity + ELEMENTS_PER_PAGE - 1) / ELEMENTS_PER_PAGE;
		let vec = Self {
			pages: (0..needed_pages)
				.map(|_| UnsafeCell::new(None))
				.collect::<Vec<_>>()
				.into_boxed_slice(),
			len: AtomicUsize::new(0),
		};
		let memory = mem::size_of::<UnsafeCell<Option<Page<T>>>>() * needed_pages;
		(vec, memory, 1)
	}

	/// Returns the number of elements stored in the vector.
	pub(super) fn len(&self) -> usize {
		self.len.load(atomic::Ordering::Acquire)
	}

	/// Returns `true` if the vector is empty.
	#[allow(unused)]
	pub(super) fn is_empty(&self) -> bool {
		self.len() == 0
	}

	/// Returns the number of used pages.
	pub(super) fn num_pages(&self) -> usize {
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
	unsafe fn page(&self, index: usize) -> &mut Option<Page<T>> {
		let page_index = Self::page_index(index);
		assert!(page_index < self.pages.len());
		&mut *self.pages.get_unchecked(page_index).get()
	}

	// Returning `mut` is allowed because of `UnsafeCell`
	#[allow(clippy::mut_from_ref)]
	unsafe fn page_or_create(&self, index: usize) -> (&mut Page<T>, usize, usize) {
		let page = self.page(index);
		if let Some(page) = page {
			(page, 0, 0)
		} else {
			let _ = Option::replace(page, Box::new(mem::zeroed()));
			(
				page.as_mut().unwrap_or_else(|| {
					debug_assert!(false, "page was not created");
					hint::unreachable_unchecked();
				}),
				mem::size_of::<Option<*const T>>() * ELEMENTS_PER_PAGE,
				1,
			)
		}
	}

	unsafe fn element(page: &mut Page<T>, index: usize) -> &mut Option<*const T> {
		debug_assert!(index < ELEMENTS_PER_PAGE);
		page.get_unchecked_mut(index)
	}

	/// Appends an element to the back of the vector.
	///
	/// # Safety
	///
	/// This is unsafe because pushing to the collection is not thread safe.
	#[allow(clippy::cast_possible_truncation)]
	pub(super) unsafe fn push(&self, value: *const T) -> (StringId, usize, usize) {
		let index = self.len.load(atomic::Ordering::Relaxed);

		let (page, memory, chunks) = self.page_or_create(index);
		let element = Self::element(page, Self::element_index(index));
		debug_assert!(element.is_none());
		let _ = Option::replace(element, value);

		self.len.store(index + 1, atomic::Ordering::Release);
		(StringId::from_raw_parts(index as u32), memory, chunks)
	}

	/// Returns the pointer at the given index, without doing bounds checking.
	pub(super) unsafe fn get_unchecked(&self, id: StringId) -> *const T {
		let id = id.get() as usize;
		let page = self.page(id).as_mut().unwrap_or_else(|| {
			debug_assert!(false, "page was not created");
			hint::unreachable_unchecked();
		});
		Self::element(page, Self::element_index(id)).unwrap_or_else(|| {
			debug_assert!(false, "element does not exist");
			hint::unreachable_unchecked();
		})
	}

	/// Returns the pointer at the given index or [`None`] if the index is out of bound.
	pub(super) fn get(&self, id: StringId) -> Option<*const T> {
		if (id.get() as usize) < self.len() {
			unsafe { Some(self.get_unchecked(id)) }
		} else {
			None
		}
	}
}

impl<T> Drop for StaticRefVector<T> {
	fn drop(&mut self) {
		let pages = self.num_pages();
		unsafe {
			for page in self.pages.iter_mut().take(pages) {
				let _ = (*page.get()).take();
			}
		}
	}
}

unsafe impl<T> Send for StaticRefVector<T> {}
unsafe impl<T> Sync for StaticRefVector<T> {}
