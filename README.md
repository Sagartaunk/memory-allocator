# memory-allocator

memory-allocator is a free list allocator with boundary tags and coalescing.

A heap memory allocator written in Rust from scratch, implementing `GlobalAlloc` to replace the system allocator.

---

## How does this allocator work ?

First we declare an arbitrary value in this case **1 MB (1024 * 1024 bytes)** as the initial storage for the allocator through the `init` function.  

Then the `init()` function calls the `heap_grow()` function from `heap.rs` which asks the OS for a chunk of free storage anywhere in memory.  

Once the storage has been granted, `heap_grow()` returns a memory pointer which is then used to create an `Allocator` struct.

The `Allocator` stores 3 memory pointers:

- `heap_start` → start of the heap  
- `heap_end` → end of the heap  
- `free_head_start` → head of the explicit free list  

`heap_end` is calculated by adding the total size to `heap_start`.

After this:

- A `BlockHeader` is written at the start and end of the heap  
- A `FreeHeader` is initialized right after the first `BlockHeader` to set up the free list  

---

## BlockHeader

The `BlockHeader` stores a single `usize` because we need to write a fixed-size word directly into memory.

It is used as follows:

- The **least significant bit (LSB)** stores allocation status  
  - `1` → allocated  
  - `0` → free  
- The remaining bits store the **size of the block**

### Methods

- `size()` → returns block size  
- `is_allocated()` → checks allocation status  
- `set_allocated(bool)` → sets allocation flag  
- `write_to(ptr)` → writes header to memory  
- `read_from(ptr)` → reads header from memory  

---

## FreeHeader

The `FreeHeader` is used to maintain an **explicit doubly linked free list**.

It is stored immediately after the `BlockHeader` in free blocks.

### Structure

- `prev` → pointer to previous free block  
- `next` → pointer to next free block  

### Purpose

- Enables traversal of only free blocks  
- Avoids scanning the entire heap  
- Improves allocation performance  

---

## Allocation Flow

When memory is requested:

1. Requested size is **aligned to 8 bytes**
2. Start traversal from `free_head_start`
3. Iterate through the **explicit free list**
4. Find a block large enough
5. Remove it from free list using `remove_free()`
6. Mark block as allocated
7. Split block if it's larger than required
8. Return pointer **after the header**

This avoids scanning allocated blocks and improves efficiency.

---

## Deallocation Flow

When freeing memory:

1. `dealloc(ptr)` is called
2. BlockHeader is marked as **free**
3. `coalesce()` is called

### Coalescing Cases

- both neighbors allocated → just mark free and insert  
- previous free, next allocated → merge with previous  
- previous allocated, next free → merge with next  
- both free → merge with both  

After merging:

- Adjacent free blocks are removed from free list  
- A new larger block is formed  
- Block is inserted back using `insert_free()`  

---

## Project Structure
src/
├── block.rs # BlockHeader + FreeHeader
├── heap.rs # mmap wrapper
├── allocator.rs # explicit free list, alloc, dealloc, split, coalesce
└── main.rs # usage example

---

## Program Flow
→ GlobalAlloc (MyAllocator)
→ Mutex
→ traverse free list
→ read BlockHeader
→ find free block
→ remove from free list
→ split (if needed)
→ mark allocated
→ return pointer
User uses memory
dealloc(ptr)
→ mark free
→ coalesce
→ insert into free list
→ reusable heap


## Known Limitations

- Unsafe previous block access  
- No heap boundary guards  
- No pointer validation in dealloc (can cause invalid free or double free)  

---

## Note:
This was a learning project and will recieve no further updates. 
Fixing the current limitations would require major rewrites to the core logic of this project,  
which is out of scope for this project.

