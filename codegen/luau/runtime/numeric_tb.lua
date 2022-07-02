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

local table_freeze = table.freeze

local from_u32, from_u64, into_u64
local num_subtract, num_divide_unsigned, num_negate
local num_or, num_shift_left
local num_is_negative, num_is_zero, num_is_less_unsigned

function Numeric.from_u32(data_1, data_2)
	return table_freeze({ data_1, data_2 })
end

function Numeric.into_u32(data)
	return data[1], data[2]
end

function Numeric.from_u64(value)
	return from_u32(bit_and(value), bit_and(value / 0x100000000))
end

function Numeric.into_u64(value)
	return value[1] + value[2] * 0x100000000
end

function Numeric.add(lhs, rhs)
	local data_1 = lhs[1] + rhs[1]
	local data_2 = lhs[2] + rhs[2]

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
	local data_1 = lhs[1] - rhs[1]
	local data_2 = lhs[2] - rhs[2]

	if data_1 < 0 then
		data_1 = data_1 + 0x100000000
		data_2 = data_2 - 1
	end

	if data_2 < 0 then
		data_2 = data_2 + 0x100000000
	end

	return from_u32(data_1, data_2)
end

function Numeric.multiply(lhs, rhs)
	if num_is_zero(lhs) or num_is_zero(rhs) then
		return NUM_ZERO
	elseif num_is_less_unsigned(lhs, NUM_BIT_26) and num_is_less_unsigned(rhs, NUM_BIT_26) then
		return from_u64(lhs[1] * rhs[1])
	end

	-- Divide each long into 4 chunks of 16 bits, and then add up 4x4 products.
	-- We can skip products that would overflow.
	local a48 = bit_rshift(lhs[2], 16)
	local a32 = bit_and(lhs[2], 0xFFFF)
	local a16 = bit_rshift(lhs[1], 16)
	local a00 = bit_and(lhs[1], 0xFFFF)

	local b48 = bit_rshift(rhs[2], 16)
	local b32 = bit_and(rhs[2], 0xFFFF)
	local b16 = bit_rshift(rhs[1], 16)
	local b00 = bit_and(rhs[1], 0xFFFF)

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
		return NUM_ZERO, NUM_ZERO
	elseif num_is_less_unsigned(lhs, NUM_BIT_52) and num_is_less_unsigned(rhs, NUM_BIT_52) then
		local lhs_u = into_u64(lhs)
		local rhs_u = into_u64(rhs)

		return from_u64(lhs_u / rhs_u), from_u64(lhs_u % rhs_u)
	end

	local quotient = NUM_ZERO
	local remainder = NUM_ZERO

	for i = 63, 0, -1 do
		local temp = num_shift_left(remainder, NUM_ONE)
		local rem_1, rem_2 = temp[1], temp[2]

		if i > 31 then
			rem_1 = bit_or(rem_1, bit_extract(lhs[2], i - 32, 1))
		else
			rem_1 = bit_or(rem_1, bit_extract(lhs[1], i, 1))
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
	local left_negative = num_is_negative(lhs)
	local right_negative = num_is_negative(rhs)

	if left_negative then
		lhs = num_negate(lhs)
	end

	if right_negative then
		rhs = num_negate(rhs)
	end

	local quotient, remainder = num_divide_unsigned(lhs, rhs)

	if left_negative ~= right_negative then
		quotient = num_negate(quotient)
	end

	if left_negative then
		remainder = num_negate(remainder)
	end

	return quotient, remainder
end

function Numeric.negate(value)
	local data_1 = bit_not(value[1]) + 1
	local data_2 = bit_not(value[2])

	if data_1 >= 0x100000000 then
		data_1 = data_1 - 0x100000000
		data_2 = data_2 + 1
	end

	if data_2 >= 0x100000000 then
		data_2 = data_2 - 0x100000000
	end

	return from_u32(data_1, data_2)
end

function Numeric.bit_and(lhs, rhs)
	local data_1 = bit_and(lhs[1], rhs[1])
	local data_2 = bit_and(lhs[2], rhs[2])

	return from_u32(data_1, data_2)
end

function Numeric.bit_not(value)
	local data_1 = bit_not(value[1])
	local data_2 = bit_not(value[2])

	return from_u32(data_1, data_2)
end

function Numeric.bit_or(lhs, rhs)
	local data_1 = bit_or(lhs[1], rhs[1])
	local data_2 = bit_or(lhs[2], rhs[2])

	return from_u32(data_1, data_2)
end

function Numeric.bit_xor(lhs, rhs)
	local data_1 = bit_xor(lhs[1], rhs[1])
	local data_2 = bit_xor(lhs[2], rhs[2])

	return from_u32(data_1, data_2)
end

function Numeric.shift_left(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local pad = 32 - count

		local data_1 = bit_lshift(lhs[1], count)
		local data_2 = bit_replace(bit_rshift(lhs[1], pad), lhs[2], count, pad)

		return from_u32(data_1, data_2)
	else
		return from_u32(0, bit_lshift(lhs[1], count - 32))
	end
end

function Numeric.shift_right_unsigned(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local data_1 = bit_replace(bit_rshift(lhs[1], count), lhs[2], 32 - count, count)
		local data_2 = bit_rshift(lhs[2], count)

		return from_u32(data_1, data_2)
	else
		return from_u32(bit_rshift(lhs[2], count - 32), 0)
	end
end

function Numeric.shift_right_signed(lhs, rhs)
	local count = into_u64(rhs)

	if count < 32 then
		local data_1 = bit_replace(bit_rshift(lhs[1], count), lhs[2], 32 - count, count)
		local data_2 = bit_arshift(lhs[2], count)

		return from_u32(data_1, data_2)
	else
		local data_1 = bit_arshift(lhs[2], count - 32)
		local data_2 = lhs[2] > 0x80000000 and 0xFFFFFFFF or 0

		return from_u32(data_1, data_2)
	end
end

function Numeric.is_negative(value)
	return value[2] >= 0x80000000
end

function Numeric.is_zero(value)
	return value[1] == 0 and value[2] == 0
end

function Numeric.is_equal(lhs, rhs)
	return lhs[1] == rhs[1] and lhs[2] == rhs[2]
end

function Numeric.is_less_unsigned(lhs, rhs)
	local lhs_2 = lhs[2]
	local rhs_2 = rhs[2]

	return lhs_2 < rhs_2 or (lhs_2 == rhs_2 and lhs[1] < rhs[1])
end

function Numeric.is_greater_unsigned(lhs, rhs)
	local lhs_2 = lhs[2]
	local rhs_2 = rhs[2]

	return lhs_2 > rhs_2 or (lhs_2 == rhs_2 and lhs[1] > rhs[1])
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
from_u64 = Numeric.from_u64
into_u64 = Numeric.into_u64

num_subtract = Numeric.subtract
num_divide_unsigned = Numeric.divide_unsigned
num_negate = Numeric.negate

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
