# memory-allocator is a free list allocator with bounday tags and coalescing 
A heap memory allocator written in Rust from scratch, implementing GlobalAlloc to replace the system allocator.  

## How does this allocator work ? 
First we declare an arbitrary value in this case 4KB or 4096 bytes as the initial storage for the allocator though the 'init' function,  
then the 'init()' function calls the 'heap_grow()' function from heap.rs which asks the os for a chunk of free storage anywhere in the memory.  
Once the Storage has been granted by the OS the 'heap_grow()' function then returns a memory pointer to that storage which the 'init()' function  
then uses to create an 'Allocator' struct which stores 2 memory pointers 'heap_start' and 'heap_end' which as the name suggests hold pointers to  
the start and end of the memory chunk. The 'heap_end' in this case is calculated by adding the number of bytes in the chunk to the 'heap_start' pointer,  
After which we write a 'BlockHeader' to the start and end of the chunk.  
The 'BlockHeader' type holds a single usize because when writing headers directly into raw memory we need to write a single word at a fixed offset  
the word is then used in the following manner :  
-> The first bit of the usize is either 0 or 1 which tells us if the block is allocated or not and is discerned by the 'is_allocated' method and managed by  
   'set_allocated()' method which takes in either a true or false value and assigns the bit as such 
-> The rest of the 'size' contains the actual size of the block and can be accessed by using the 'size()' method implemented on 'BlockHeader' struct 
-> Then there are two methods namely   
  'write_to()' -> Writes the Block to a given memory pointer 
  'read_from()' -> reads the BlockHeader from a given location in the memory
The 'GlobalAlloc' Struct is a 'zero struct' which means it does not have any size and is used as a wrapper for Allocator struct as well as declared the default allocator.  

### What exactly happens when you call for a block of memory to be allocated ?
Now, the Allocator already holds a chunk of 4 KB but we cannot assign the whole thing to something that might need a fraction of that memory, so when something calls for memory to be allocated to it we check how many bytes it needs, and allign it to a multiple of 8 bytes and then split the initial 4KB blocks into 2 blocks one which has the amount of memory the allocation needs and the other a free block and write headers and footers in both. 
For consicutive allocation we iterate over the allocated blocks (by adding their size to their memory pointer) and when we find one which is larger than we need we split it. This in turn saves memory as well as provides a fast way to iterate over blocks because of the BlockHeaders.

### What happens when we ask for deallocating a chunk of memory ? 
We call the 'dealloc()' method with the pointer to the memory we want to free , the 'dealloc' method then simple flips the last bit to 0 in the BlockHeader and rewrites it marking it as free block. Then comes in the 'coalesce' function which merges it with adjacent free blocks to prevent fragmentation. There are 4 cases in this :  
both neighbors allocated → do nothing, just mark free  
previous free, next allocated → merge with previous  
previous allocated, next free → merge with next  
both free → merge with both  

## Project Structure 
src/
  block.rs    - BlockHeader struct, bit packing, read/write primitives
  heap.rs     - mmap wrapper to acquire raw memory from OS
  allocator.rs - free list logic, alloc, dealloc, split, coalesce, GlobalAlloc impl
  main.rs     - usage example
  

Program → GlobalAlloc (MyAllocator) → Mutex<Allocator> → scan heap_start → read BlockHeader → find free block → split (if large) → mark allocated → return ptr (after header) → user uses memory → dealloc(ptr) → mark free → coalesce adjacent blocks → reusable heap



## Known Limitations 
-> Unsafe previous block access  
-> No heap boundary guards  
-> No pointer validation in dealloc (might lead to double free or an invalid free)  
-> Uses Implicit free list


## TO-Do
-> Make an explicit free list
