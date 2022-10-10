use super::{add_padding, remove_padding};

#[test]
fn pad_add_remove() {
  use rand::Rng;
  const TESTS: usize = 20;
  for _ in 0..TESTS {
    let mut rng = rand::thread_rng();
    let data_len: usize = rng.gen_range(1000..2000);
    let chunk_len: usize = rng.gen_range(1..10);

    let pad_value: u8 = rng.gen();
    let data: Vec<u8> = rng.sample_iter(rand::distributions::Standard).take(data_len).collect();
    let mut data_pad = data.clone();
    add_padding(&mut data_pad, pad_value, chunk_len);
    remove_padding(&mut data_pad, data_len, chunk_len);
    assert_eq!(data_pad, data)
  }
}
