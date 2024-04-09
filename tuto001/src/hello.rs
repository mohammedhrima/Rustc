
use std::io;
use rand::Rng;
use std::io::{Write, BufReader, BufRead, ErrorKind};
use std::fs::File;
use std::cmp::Ordering;

pub fn hello()
{
    println!("Enter your name: ");
    let mut name:String = String::new();
    let greeting: &str = "Nice to meet you";
    io::stdin().read_line(&mut name).expect("Didn't receive Input");
    println!("Hello {}, {}", name.trim_end(), greeting);
}