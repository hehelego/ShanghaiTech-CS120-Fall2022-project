/// add padding at the beginning of the sequence,
/// the padding section are filled with `pad_value`.
/// make sure that `seq.len() % chunk_len == 0`
pub fn add_padding<T: Clone>(seq: &mut Vec<T>, pad_value: T, chunk_len: usize) {
  let padding_len = (chunk_len - seq.len() % chunk_len) % chunk_len;
  seq.extend(std::iter::repeat(pad_value).take(padding_len));
}
/// remove the padding section in front of the original data.
/// the length of the original sequence and the padding chunk len should be given.
pub fn remove_padding<T: Clone>(seq: &mut Vec<T>, original_len: usize, chunk_len: usize) {
  let _ = chunk_len;
  seq.resize(original_len, seq[0].clone());
}
