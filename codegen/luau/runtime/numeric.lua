local Numeric = {}

local BIT_SET_31 = 0x80000000
local BIT_SET_32 = 0x100000000

local K_ZERO, K_ONE, K_BIT_SET_26

local bit_lshift = bit32.lshift
local bit_rshift = bit32.rshift
local bit_arshift = bit32.arshift

local bit_and = bit32.band
local bit_or = bit32.bor
local bit_xor = bit32.bxor
local bit_not = bit32.bnot

local bit_replace = bit32.replace

local math_ceil = math.ceil
local math_floor = math.floor
local math_log = math.log
local math_max = math.max
local math_pow = math.pow

local table_freeze = table.freeze

local from_u32, into_u32, from_u64, into_u64
local num_add, num_subtract, num_multiply, num_divide_unsigned, num_negate, num_not
local num_is_negative, num_is_zero, num_is_equal, num_is_less_unsigned, num_is_greater_unsigned

-- TODO: Eventually support Vector3
function Numeric.from_u32(data_1, data_2)
	return table_freeze({ data_1, data_2 })
end

function Numeric.into_u32(data)
	return data[1], data[2]
end

function Numeric.from_u64(value)
	return from_u32(bit_and(value), math_floor(value / BIT_SET_32))
end

function Numeric.into_u64(value)
	local data_1, data_2 = into_u32(value)

	return data_1 + data_2 * BIT_SET_32
end

function Numeric.add(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	local data_1 = data_l_1 + data_r_1
	local data_2 = data_l_2 + data_r_2

	if data_1 >= BIT_SET_32 then
		data_1 = data_1 - BIT_SET_32
		data_2 = data_2 + 1
	end

	if data_2 >= BIT_SET_32 then
		data_2 = data_2 - BIT_SET_32
	end

	return from_u32(data_1, data_2)
end

function Numeric.subtract(lhs, rhs)
	local data_l_1, data_l_2 = into_u32(lhs)
	local data_r_1, data_r_2 = into_u32(rhs)

	local data_1 = data_l_1 - data_r_1
	local data_2 = data_l_2 - data_r_2

	if data_1 < 0 then
		data_1 = data_1 + BIT_SET_32
		data_2 = data_2 - 1
	end

	if data_2 < 0 then
		data_2 = data_2 + BIT_SET_32
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
		return K_ZERO
	end

	local has_negative

	has_negative, lhs, rhs = set_absolute(lhs, rhs)

	-- If both longs are small, use float multiplication
	if num_is_less_unsigned(lhs, K_BIT_SET_26) and num_is_less_unsigned(rhs, K_BIT_SET_26) then
		local data_l_1, _ = into_u32(lhs)
		local data_r_1, _ = into_u32(rhs)
		local result = from_u64(data_l_1 * data_r_1)

		if has_negative then
			result = num_negate(result)
		end

		return result
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
	local result = from_u32(data_1, data_2)

	if has_negative then
		result = num_negate(result)
	end

	return result
end

local function get_approx_delta(rem, rhs)
	local approx = math_max(1, math_floor(rem / rhs))
	local log = math_ceil(math_log(approx, 2))
	local delta = log <= 48 and 1 or math_pow(2, log - 48)

	return approx, delta
end

function Numeric.divide_unsigned(lhs, rhs)
	if num_is_zero(rhs) then
		error("division by zero")
	elseif num_is_zero(lhs) then
		return 0
	end

	local rhs_number = into_u64(rhs)
	local rem = lhs
	local res = K_ZERO

	while num_is_greater_unsigned(rem, rhs) or num_is_equal(rem, rhs) do
		local res_approx, delta = get_approx_delta(into_u64(rem), rhs_number)
		local res_temp = from_u64(res_approx)
		local rem_temp = num_multiply(res_temp, rhs)

		while num_is_negative(rem_temp) or num_is_greater_unsigned(rem_temp, rem) do
			res_approx = res_approx - delta
			res_temp = from_u64(res_approx)
			rem_temp = num_multiply(res_temp, rhs)
		end

		if num_is_zero(res_temp) then
			res_temp = K_ONE
		end

		res = num_add(res, res_temp)
		rem = num_subtract(rem, rem_temp)
	end

	return res
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
	return num_add(num_not(value), K_ONE)
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
		local data_2 = data_l_2 > BIT_SET_31 and BIT_SET_32 - 1 or 0

		return from_u32(data_1, data_2)
	end
end

function Numeric.is_negative(value)
	local _, data_2 = into_u32(value)

	return data_2 >= BIT_SET_31
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
num_multiply = Numeric.multiply
num_divide_unsigned = Numeric.divide_unsigned
num_negate = Numeric.negate
num_not = Numeric.bit_not

num_is_negative = Numeric.is_negative
num_is_zero = Numeric.is_zero
num_is_equal = Numeric.is_equal
num_is_less_unsigned = Numeric.is_less_unsigned
num_is_greater_unsigned = Numeric.is_greater_unsigned

K_ZERO = from_u64(0)
K_ONE = from_u64(1)
K_BIT_SET_26 = from_u64(0x4000000)

Numeric.K_ZERO = K_ZERO
Numeric.K_ONE = K_ONE

return table_freeze(Numeric)
