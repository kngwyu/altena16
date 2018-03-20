s = "const BLEND_TABLE: [[u8; 256]; 16] = ["
16.times do |i|
  s += "["
  256.times do |v|
    ratio = (i - 0.0) / 15.0
    value = (v - 0.0) * ratio
    s += value.round.to_i.to_s + ", "
  end
  s += "],"
end
puts s + "];"
