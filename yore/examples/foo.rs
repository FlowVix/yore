use yore::{Seer, Yorer};

fn main() {
    let mut yorer = Yorer::new(0);
    macro_rules! h {
        () => {
            println!("{}", *yorer);
        };
    }
    h!();
    yorer.seer().set(1);
    h!();
    yorer.seer().set(2);
    h!();
    yorer.undo();
    h!();
    yorer.redo();
    h!();
    yorer.undo();
    h!();
    yorer.undo();
    h!();
    yorer.redo();
    h!();
}
