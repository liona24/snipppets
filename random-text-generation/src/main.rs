use std::io;
use std::collections::HashMap;
use rand::{thread_rng, Rng};

fn read_words() -> Vec<String> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf).expect("Could not read stdin!");
    buf.split(" ").map(|x| x.to_ascii_lowercase()).collect()
}


fn build_state_table(words : Vec<String>) -> HashMap<String, Vec<String>> {
    let mut table = HashMap::new();

    let mut p_word = "\n".to_string();
    let mut pp_word = "\n".to_string();
    for x in words {
        let key = format!("{} {}", pp_word, p_word);

        table.entry(key)
            .or_insert(Vec::new())
            .push(x.clone());

        pp_word = p_word;
        p_word = x;
    }
    let key = format!("{} {}", pp_word, p_word);
    table.entry(key)
        .or_insert(Vec::new())
        .push("\n".to_string());

    table
}


fn main() {
    println!("Enter a bunch of words (space seperated):");
    let words = read_words();

    let states = build_state_table(words);

    let mut p_word = &"\n".to_string();
    let mut pp_word = &"\n".to_string();

    println!("--------------------------------------------------------------------");

    let mut rng = thread_rng();
    for _ in 0..20 {
        if let Some(next_values) = states.get(&format!("{} {}", pp_word, p_word)) {
            let i = rng.gen_range(0, next_values.len());
            let next_word = &next_values[i];

            print!("{} ", next_word);

            pp_word = p_word;
            p_word = next_word;
        } else {
            println!("");
            break;
        }
    }
}
