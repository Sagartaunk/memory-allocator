use memory_allocator::allocator::MyAllocator;

#[global_allocator]
pub static MY_ALLOCATOR: MyAllocator = MyAllocator; //Static to tell rust which allocator to use

fn main() {
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    println!("{:?}", v);
}
