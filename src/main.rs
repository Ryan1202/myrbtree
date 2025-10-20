use std::time::Instant;
use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;
use avltree;

pub mod my_rbtree;

fn main() {
    // 不依赖输入参数：使用固定大小和固定种子，先输出生成的序列
    let n: usize = 200;

    // 固定种子，不从命令行读取
    let seed: u64 = 0x12345678;
    let mut rng = StdRng::seed_from_u64(seed);

    // 生成并打乱序列，先输出生成的序列（为避免过大输出，这里仅显示前100个）
    let mut keys: Vec<i32> = (0..n).map(|i| i as i32).collect();
    keys.shuffle(&mut rng);
    println!("Generated sequence (first {} of {}): {:?}", 100.min(n), n, &keys[..100.min(n)]);

    let mut keys = vec![1, 9, 2, 8, 3, 7, 4, 6, 5];
    // create tree and measure insertion time
    // Run insertion in a separate thread with a larger stack to avoid stack overflow
    let keys_clone = keys.clone();
    let handle = std::thread::Builder::new()
        .name("inserter".into())
        .spawn(move || {
            let mut rbtree = rbtree::RBTree::new();
            let mut tree: my_rbtree::RbTree<i32> = my_rbtree::RbTree::new();
            let mut avl_tree = avltree::SearchTree::new();

            let start = Instant::now();
            for &k in &keys_clone {
                tree.insert(k);
            }
            let elapsed = start.elapsed();
            println!("Inserted {} keys in {:?} for my RBTree", n, elapsed);
            
            let start = Instant::now();
            for &k in &keys_clone {
                rbtree.insert(k, k);
            }
            let elapsed = start.elapsed();
            println!("Inserted {} keys in {:?} for RBTree", n, elapsed);

            let start = Instant::now();
            for &k in &keys_clone {
                avl_tree.insert(k);
            }
            let elapsed = start.elapsed();
            println!("Inserted {} keys in {:?} for AVLTree", n, elapsed);

            keys.shuffle(&mut rng);

            let start = Instant::now();
            let mut start2;
            let mut elapsed2;
            let mut max = Instant::now().elapsed();
            for &k in &keys_clone {
                start2 = Instant::now();
                tree.get(&k);
                elapsed2 = start2.elapsed();
                max = elapsed2;
                if elapsed2 > max {
                    max = elapsed2;
                }
            }
            let elapsed = start.elapsed();
            println!("Searched {} keys in {:?} for my RBTree.Max {:?}", n, elapsed, max);


            let start = Instant::now();
            let mut start2;
            let mut elapsed2;
            let mut max = Instant::now().elapsed();
            for &k in &keys_clone {
                start2 = Instant::now();
                rbtree.get(&k);
                elapsed2 = start2.elapsed();
                max = elapsed2;
                if elapsed2 > max {
                    max = elapsed2;
                }
            }
            let elapsed = start.elapsed();
            println!("Searched {} keys in {:?} for RBTree.Max {:?}", n, elapsed, max);

            let start = Instant::now();
            let mut start2;
            let mut elapsed2;
            let mut max = Instant::now().elapsed();
            for &k in &keys_clone {
                start2 = Instant::now();
                avl_tree.contains(&k);
                elapsed2 = start2.elapsed();
                max = elapsed2;
                if elapsed2 > max {
                    max = elapsed2;
                }
            }
            let elapsed = start.elapsed();
            println!("Searched {} keys in {:?} for AVLTree.Max {:?}", n, elapsed, max);
        })
        .expect("failed to spawn inserter thread");

    handle.join().expect("inserter thread panicked");
}
