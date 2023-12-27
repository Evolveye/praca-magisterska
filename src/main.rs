fn tuple() -> (i32, i32) {
    (1, 2)
}

fn main() {
    println!("Hello, world!");
    let mut nums = vec![1, 2, 3, 4];

    nums.push(5);

    for i in 0..10_000 {
        nums.push(i * 10)
    }

    println!("{:?}", &nums[0..10]);
}
