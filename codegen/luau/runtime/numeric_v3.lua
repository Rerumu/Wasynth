local Numeric = {}

local NUM_ZERO, NUM_ONE, NUM_BIT_26, NUM_BIT_52

local bit_lshift = bit32.lshift
local bit_rshift = bit32.rshift
local bit_arshift = bit32.arshift

local bit_and = bit32.band
local bit_or = bit32.bor
local bit_xor = bit32.bxor
local bit_not = bit32.bnot

local bit_extract = bit32.extract
local bit_replace = bit32.replace

local math_floor = math.floor

local table_freeze = table.freeze

local from_u32, into_u32, from_u64, into_u64
local num_add, num_subtract, num_divide_unsigned, num_negate
local num_not, num_or, num_shift_left
local num_is_negative, num_is_zero, num_is_less_unsigned

local constructor = Vector3.new

-- X: a[0 ..21]
-- Y: a[22..31]
--  | b[0 ..11]
-- Z: b[12..31]

function Numeric.from_u32(data_1, data_2)
	local x = bit_and(data_1, 0x3FFFFF)
	local y = bit_or(bit_and(data_1, 0xFFC00000), bit_and(data_2, 0xFFF))
	local z = bit_extract(data_2, 12, 20)

	return constructor(x, y, z)
end

function Numeric.into_u32(data)
	local data_1 = bit_or(bit_and(data.X, 0x3FFFFF), bit_and(data.Y, 0xFFC00000))
	local data_2 = bit_replace(bit_and(data.Y, 0xFFF), data.Z, 12, 20)

	return data_1, data_2
end

function Numeric.from_u64(value)
	return from_u32(bit_and(value), math_floor(value / 0x100000000))
end

function Numeric.into_u64(value)
	local data_1, data_2 = into_u32(value)

	return data_1 + data_2 * 0x100000000
end

function Numeric.add(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	local data_1 = data_l_1 + data_r_1
	local data_2 = data_l_2 + data_r_2

	if data_1 >= 0x100000000 then
		data_1 = data_1 - 0x100000000
		data_2 = data_2 + 1
	end

	if data_2 >= 0x100000000 then
		data_2 = data_2 - 0x100000000
	end

	return from_u32(data_1, data_2)
end

function Numeric.subtract(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	local data_1 = data_l_1 - data_r_1
	local data_2 = data_l_2 - data_r_2

	if data_1 < 0 then
		data_1 = data_1 + 0x100000000
		data_2 = data_2 - 1
	end

	if data_2 < 0 then
		data_2 = data_2 + 0x100000000
	end

	return from_u32(data_1, data_2)
end

local function set_absolute(lhs, rhs)
	local has_negative = false

	if num_is_negative(lhs) then
		lhs = num_negate(lhs)
		has_negative = not has_negative
	end

	if num_is_negative(rhs) then
		rhs = num_negate(rhs)
		has_negative = not has_negative
	end

	return has_negative, lhs, rhs
end

function Numeric.multiply(lhs, rhs)
	if num_is_zero(lhs) or num_is_zero(rhs) then
		return NUM_ZERO
	end

	-- If both longs are small, use float multiplication
	if num_is_less_unsigned(lhs, NUM_BIT_26) and num_is_less_unsigned(rhs, NUM_BIT_26) then
		local data_l_1, _ = into_u32(lhs)
		local data_r_1, _ = into_u32(rhs)

		return from_u64(data_l_1 * data_r_1)
	end

	-- Divide each long into 4 chunks of 16 bits, and then add up 4x4 products.
	-- We can skip products that would overflow.
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	local a48 = bit_rshift(data_l_2, 16)
	local a32 = bit_and(data_l_2, 0xFFFF)
	local a16 = bit_rshift(data_l_1, 16)
	local a00 = bit_and(data_l_1, 0xFFFF)

	local b48 = bit_rshift(data_r_2, 16)
	local b32 = bit_and(data_r_2, 0xFFFF)
	local b16 = bit_rshift(data_r_1, 16)
	local b00 = bit_and(data_r_1, 0xFFFF)

	local c00 = a00 * b00
	local c16 = bit_rshift(c00, 16)

	c00 = bit_and(c00, 0xFFFF)
	c16 = c16 + a16 * b00

	local c32 = bit_rshift(c16, 16)

	c16 = bit_and(c16, 0xFFFF)
	c16 = c16 + a00 * b16
	c32 = c32 + bit_rshift(c16, 16)
	c16 = bit_and(c16, 0xFFFF)
	c32 = c32 + a32 * b00

	local c48 = bit_rshift(c32, 16)

	c32 = bit_and(c32, 0xFFFF)
	c32 = c32 + a16 * b16
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_and(c32, 0xFFFF)
	c32 = c32 + a00 * b32
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_and(c32, 0xFFFF)
	c48 = c48 + a48 * b00 + a32 * b16 + a16 * b32 + a00 * b48
	c48 = bit_and(c48, 0xFFFF)

	local data_1 = bit_replace(c00, c16, 16, 16)
	local data_2 = bit_replace(c32, c48, 16, 16)

	return from_u32(data_1, data_2)
end

function Numeric.divide_unsigned(lhs, rhs)
	if num_is_zero(rhs) then
		error("division by zero")
	elseif num_is_zero(lhs) then
		return NUM_ZERO
	elseif num_is_less_unsigned(lhs, NUM_BIT_52) and num_is_less_unsigned(rhs, NUM_BIT_52) then
		local result = math_floor(into_u64(lhs) / into_u64(rhs))

		return from_u64(result)
	end

	local quotient = NUM_ZERO
	local remainder = NUM_ZERO

	local num_1, num_2 = into_u32(lhs)

	for i = 63, 0, -1 do
		local rem_1, rem_2 = into_u32(num_shift_left(remainder, NUM_ONE))

		if i > 31 then
			rem_1 = bit_or(rem_1, bit_extract(num_2, i - 32, 1))
		else
			rem_1 = bit_or(rem_1, bit_extract(num_1, i, 1))
		end

		remainder = from_u32(rem_1, rem_2)

		if not num_is_less_unsigned(remainder, rhs) then
			remainder = num_subtract(remainder, rhs)
			quotient = num_or(quotient, num_shift_left(NUM_ONE, from_u32(i, 0)))
		end
	end

	return quotient, remainder
end

function Numeric.divide_signed(lhs, rhs)
	local has_negative

	has_negative, lhs, rhs = set_absolute(lhs, rhs)

	local result = num_divide_unsigned(lhs, rhs)

	if has_negative then
		result = num_negate(result)
	end

	return result
end

function Numeric.negate(value)
	return num_add(num_not(value), NUM_ONE)
end

function Numeric.bit_and(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return from_u32(bit_and(data_l_1, data_r_1), bit_and(data_l_2, data_r_2))
end

function Numeric.bit_not(value)
	local data_1, data_2 = into_u32(value)

	return from_u32(bit_not(data_1), bit_not(data_2))
end

function Numeric.bit_or(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return from_u32(bit_or(data_l_1, data_r_1), bit_or(data_l_2, data_r_2))
end

function Numeric.bit_xor(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return from_u32(bit_xor(data_l_1, data_r_1), bit_xor(data_l_2, data_r_2))
end

function Numeric.shift_left(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local pad = 32 - count
		local data_l_1, data_l_2 = into_u32(lhs)

		local data_1 = bit_lshift(data_l_1, count)
		local data_2 = bit_replace(bit_rshift(data_l_1, pad), data_l_2, count, pad)

		return from_u32(data_1, data_2)
	elseif count == 32 then
		local data_l_1, _ = into_u32(lhs)

		return from_u32(0, data_l_1)
	else
		local data_l_1, _ = into_u32(lhs)

		return from_u32(0, bit_lshift(data_l_1, count - 32))
	end
end

function Numeric.shift_right_unsigned(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local data_l_1, data_l_2 = into_u32(lhs)

		local data_1 = bit_replace(bit_rshift(data_l_1, count), data_l_2, 32 - count, count)
		local data_2 = bit_rshift(data_l_2, count)

		return from_u32(data_1, data_2)
	elseif count == 32 then
		local _, data_l_2 = into_u32(lhs)

		return from_u32(data_l_2, 0)
	else
		local _, data_l_2 = into_u32(lhs)

		return from_u32(bit_rshift(data_l_2, count - 32), 0)
	end
end

function Numeric.shift_right_signed(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local data_l_1, data_l_2 = into_u32(lhs)

		local data_1 = bit_replace(bit_rshift(data_l_1, count), data_l_2, 32 - count, count)
		local data_2 = bit_arshift(data_l_2, count)

		return from_u32(data_1, data_2)
	else
		local _, data_l_2 = into_u32(lhs)

		local data_1 = bit_arshift(data_l_2, count - 32)
		local data_2 = data_l_2 > 0x80000000 and 0xFFFFFFFF or 0

		return from_u32(data_1, data_2)
	end
end

function Numeric.is_negative(value)
	local _, data_2 = into_u32(value)

	return data_2 >= 0x80000000
end

function Numeric.is_zero(value)
	local data_1, data_2 = into_u32(value)

	return data_1 == 0 and data_2 == 0
end

function Numeric.is_equal(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return data_l_1 == data_r_1 and data_l_2 == data_r_2
end

function Numeric.is_less_unsigned(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return data_l_2 < data_r_2 or (data_l_2 == data_r_2 and data_l_1 < data_r_1)
end

function Numeric.is_greater_unsigned(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	return data_l_2 > data_r_2 or (data_l_2 == data_r_2 and data_l_1 > data_r_1)
end

function Numeric.is_less_signed(lhs, rhs)
	local neg_a = num_is_negative(lhs)
	local neg_b = num_is_negative(rhs)

	if neg_a and not neg_b then
		return true
	elseif not neg_a and neg_b then
		return false
	else
		return num_is_negative(num_subtract(lhs, rhs))
	end
end

function Numeric.is_greater_signed(lhs, rhs)
	local neg_a = num_is_negative(lhs)
	local neg_b = num_is_negative(rhs)

	if neg_a and not neg_b then
		return false
	elseif not neg_a and neg_b then
		return true
	else
		return num_is_negative(num_subtract(rhs, lhs))
	end
end

from_u32 = Numeric.from_u32
into_u32 = Numeric.into_u32
from_u64 = Numeric.from_u64
into_u64 = Numeric.into_u64

num_add = Numeric.add
num_subtract = Numeric.subtract
num_divide_unsigned = Numeric.divide_unsigned
num_negate = Numeric.negate

num_not = Numeric.bit_not
num_or = Numeric.bit_or
num_shift_left = Numeric.shift_left

num_is_negative = Numeric.is_negative
num_is_zero = Numeric.is_zero
num_is_less_unsigned = Numeric.is_less_unsigned

NUM_ZERO = from_u64(0)
NUM_ONE = from_u64(1)
NUM_BIT_26 = from_u64(0x4000000)
NUM_BIT_52 = from_u64(0x10000000000000)

Numeric.ZERO = NUM_ZERO
Numeric.ONE = NUM_ONE

return table_freeze(Numeric)
