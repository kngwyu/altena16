s = "const BLEND_TABLE: [u8; 256 * 16] = ["
256.times do |v| 
  16.times do |i|
    ratio = (i - 0.0) / 15.0
    value = (v - 0.0) * ratio
    s += value.round.to_i.to_s + ", "
  end
end
puts s + "];"
