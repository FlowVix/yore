use yore::{Seer, Yorer};

macro_rules! h {
    ($y:ident => $($do:expr;)*) => {
        $(
            $do;
            println!("{:<25}: {:?}", stringify!($do), *$y);
        )*
    };
}

fn main() {
    let mut y = Yorer::new(vec![1, 2, 3, 4, 5, 6, 7, 8]);
    println!("{:<25}: {:?}", "start", *y);

    h! {
        y =>

        y.seer().push(69);
        y.seer().pop();
        y.seer().pop();
        y.seer().push(1000);
        y.seer().truncate(2);
        y.undo();
        y.undo();
        y.undo();
        y.undo();
        y.undo();
        y.redo();
        y.redo();
        y.redo();
        y.redo();
        y.redo();
    }
}
