pub fn read_line() -> String {
  let mut input = String::new();
  std::io::stdin().read_line(&mut input).unwrap();
  return input.trim().to_string();
}