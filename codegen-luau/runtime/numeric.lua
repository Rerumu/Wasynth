local Numeric = {}

Numeric.__index = Numeric

local bit_band = bit32.band
local bit_bnot = bit32.bnot
local bit_bor = bit32.bor
local bit_xor = bit32.bxor

local bit_lshift = bit32.lshift
local bit_rshift = bit32.rshift
local bit_arshift = bit32.arshift

local math_floor = math.floor

local N_2_TO_31 = 0x80000000
local N_2_TO_32 = 0x100000000

local VAL_ZERO
local VAL_ONE
local VAL_2_TO_24

local op_is_equal
local op_is_greater_unsigned
local op_is_less_unsigned
local op_is_negative
local op_is_zero

local op_bnot
local op_negate

-- TODO: Eventually support Vector3
local function from_u32(low, high)
	return setmetatable({ low, high }, Numeric)
end

local function to_u32(value)
	return value[1], value[2]
end

local function from_f64(value)
	if value < 0 then
		return op_negate(from_f64(-value))
	else
		return from_u32(value % N_2_TO_32, math_floor(value / N_2_TO_32))
	end
end

local function to_f64(value)
	local low, high = to_u32(value)

	return low + high * N_2_TO_32
end

local function op_add(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	local low = low_a + low_b
	local high = high_a + high_b

	if low >= N_2_TO_32 then
		low = low - N_2_TO_32
		high = high + 1
	end

	if high >= N_2_TO_32 then
		high = high - N_2_TO_32
	end

	return from_u32(low, high)
end

local function op_subtract(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	local low = low_a - low_b
	local high = high_a - high_b

	if low < 0 then
		low = low + N_2_TO_32
		high = high - 1
	end

	if high < 0 then
		high = high + N_2_TO_32
	end

	return from_u32(low, high)
end

local function set_absolute(lhs, rhs)
	local has_negative = false

	if op_is_negative(lhs) then
		lhs = op_negate(lhs)
		has_negative = not has_negative
	end

	if op_is_negative(rhs) then
		rhs = op_negate(rhs)
		has_negative = not has_negative
	end

	return has_negative, lhs, rhs
end

local function op_multiply(lhs, rhs)
	if op_is_zero(lhs) or op_is_zero(rhs) then
		return VAL_ZERO
	end

	local has_negative

	has_negative, lhs, rhs = set_absolute(lhs, rhs)

	-- If both longs are small, use float multiplication
	if op_is_less_unsigned(lhs, VAL_2_TO_24) and op_is_less_unsigned(rhs, VAL_2_TO_24) then
		local low_a = to_u32(lhs)
		local low_b = to_u32(rhs)
		local result = from_f64(low_a * low_b)

		if has_negative then
			result = op_negate(result)
		end

		return result
	end

	-- Divide each long into 4 chunks of 16 bits, and then add up 4x4 products.
	-- We can skip products that would overflow.
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	local a48 = bit_rshift(high_a, 16)
	local a32 = bit_band(high_a, 0xFFFF)
	local a16 = bit_rshift(low_a, 16)
	local a00 = bit_band(low_a, 0xFFFF)

	local b48 = bit_rshift(high_b, 16)
	local b32 = bit_band(high_b, 0xFFFF)
	local b16 = bit_rshift(low_b, 16)
	local b00 = bit_band(low_b, 0xFFFF)

	local c48, c32, c16, c00 = 0, 0, 0, 0

	c00 = c00 + a00 * b00
	c16 = c16 + bit_rshift(c00, 16)
	c00 = bit_band(c00, 0xFFFF)
	c16 = c16 + a16 * b00
	c32 = c32 + bit_rshift(c16, 16)
	c16 = bit_band(c16, 0xFFFF)
	c16 = c16 + a00 * b16
	c32 = c32 + bit_rshift(c16, 16)
	c16 = bit_band(c16, 0xFFFF)
	c32 = c32 + a32 * b00
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_band(c32, 0xFFFF)
	c32 = c32 + a16 * b16
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_band(c32, 0xFFFF)
	c32 = c32 + a00 * b32
	c48 = c48 + bit_rshift(c32, 16)
	c32 = bit_band(c32, 0xFFFF)
	c48 = c48 + a48 * b00 + a32 * b16 + a16 * b32 + a00 * b48
	c48 = bit_band(c48, 0xFFFF)

	local low_v = bit_bor(bit_lshift(c16, 16), c00)
	local high_v = bit_bor(bit_lshift(c48, 16), c32)
	local result = from_u32(low_v, high_v)

	if has_negative then
		result = op_negate(result)
	end

	return result
end

local math_ceil = math.ceil
local math_log = math.log
local math_max = math.max
local math_pow = math.pow

local function get_approx_delta(rem, rhs)
	local approx = math_max(1, math_floor(rem / rhs))
	local log = math_ceil(math_log(approx, 2))
	local delta = log <= 48 and 1 or math_pow(2, log - 48)

	return approx, delta
end

local function op_divide_unsigned(lhs, rhs)
	if op_is_zero(rhs) then
		error("division by zero")
	elseif op_is_zero(lhs) then
		return 0
	end

	local rhs_number = to_f64(rhs)
	local rem = lhs
	local res = VAL_ZERO

	while op_is_greater_unsigned(rem, rhs) or op_is_equal(rem, rhs) do
		local res_approx, delta = get_approx_delta(to_f64(rem), rhs_number)
		local res_temp = from_f64(res_approx)
		local rem_temp = op_multiply(res_temp, rhs)

		while op_is_negative(rem_temp) or op_is_greater_unsigned(rem_temp, rem) do
			res_approx = res_approx - delta
			res_temp = from_f64(res_approx)
			rem_temp = op_multiply(res_temp, rhs)
		end

		if op_is_zero(res_temp) then
			res_temp = VAL_ONE
		end

		res = op_add(res, res_temp)
		rem = op_subtract(rem, rem_temp)
	end

	return res
end

local function op_divide_signed(lhs, rhs)
	local has_negative

	has_negative, lhs, rhs = set_absolute(lhs, rhs)

	local result = op_divide_unsigned(lhs, rhs)

	if has_negative then
		result = op_negate(result)
	end

	return result
end

function op_negate(value)
	return op_add(op_bnot(value), VAL_ONE)
end

local function op_band(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return from_u32(bit_band(low_a, low_b), bit_band(high_a, high_b))
end

function op_bnot(value)
	local low, high = to_u32(value)

	return from_u32(bit_bnot(low), bit_bnot(high))
end

local function op_bor(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return from_u32(bit_bor(low_a, low_b), bit_bor(high_a, high_b))
end

local function op_bxor(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return from_u32(bit_xor(low_a, low_b), bit_xor(high_a, high_b))
end

local function op_shift_left(lhs, rhs)
	local count = to_f64(rhs)

	if count < 32 then
		local low_a, high_a = to_u32(lhs)

		local low_v = bit_lshift(low_a, count)
		local high_v = bit_bor(bit_lshift(high_a, count), bit_rshift(low_a, 32 - count))

		return from_u32(low_v, high_v)
	else
		local _, high_a = to_u32(lhs)

		local high_v = bit_lshift(high_a, count - 32)

		return from_u32(0, high_v)
	end
end

local function op_shift_right_unsigned(lhs, rhs)
	local count = to_f64(rhs)

	if count < 32 then
		local low_a, high_a = to_u32(lhs)

		local low_v = bit_bor(bit_rshift(low_a, count), bit_lshift(high_a, 32 - count))
		local high_v = bit_rshift(high_a, count)

		return from_u32(low_v, high_v)
	elseif count == 32 then
		local _, high_a = to_u32(lhs)

		return from_u32(high_a, 0)
	else
		local _, high_a = to_u32(lhs)

		return from_u32(bit_rshift(high_a, count - 32), 0)
	end
end

local function op_shift_right_signed(lhs, rhs)
	local count = to_f64(rhs)

	if count < 32 then
		local low_a, high_a = to_u32(lhs)

		local low_v = bit_bor(bit_rshift(low_a, count), bit_lshift(high_a, 32 - count))
		local high_v = bit_arshift(high_a, count)

		return from_u32(low_v, high_v)
	else
		local _, high_a = to_u32(lhs)

		local low_v = bit_arshift(high_a, count - 32)
		local high_v = high_a > N_2_TO_31 and N_2_TO_32 - 1 or 0

		return from_u32(low_v, high_v)
	end
end

function op_is_negative(value)
	local _, high = to_u32(value)

	return high > N_2_TO_31
end

function op_is_zero(value)
	local low, high = to_u32(value)

	return low == 0 and high == 0
end

function op_is_equal(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return low_a == low_b and high_a == high_b
end

function op_is_less_unsigned(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return high_a < high_b or (high_a == high_b and low_a < low_b)
end

function op_is_greater_unsigned(lhs, rhs)
	local low_a, high_a = to_u32(lhs)
	local low_b, high_b = to_u32(rhs)

	return high_a > high_b or (high_a == high_b and low_a > low_b)
end

local function op_is_less_signed(lhs, rhs)
	local neg_a = op_is_negative(lhs)
	local neg_b = op_is_negative(rhs)

	if neg_a and not neg_b then
		return true
	elseif not neg_a and neg_b then
		return false
	else
		return op_is_negative(op_subtract(lhs, rhs))
	end
end

local function op_is_greater_signed(lhs, rhs)
	local neg_a = op_is_negative(lhs)
	local neg_b = op_is_negative(rhs)

	if neg_a and not neg_b then
		return false
	elseif not neg_a and neg_b then
		return true
	else
		return op_is_negative(op_subtract(rhs, lhs))
	end
end

VAL_ZERO = from_f64(0)
VAL_ONE = from_f64(1)
VAL_2_TO_24 = from_f64(0x1000000)

Numeric.from_f64 = from_f64
Numeric.from_u32 = from_u32
Numeric.to_f64 = to_f64

Numeric.divide_unsigned = op_divide_unsigned

Numeric.bit_and = op_band
Numeric.bit_not = op_bnot
Numeric.bit_or = op_bor
Numeric.bit_xor = op_bxor

Numeric.shift_left = op_shift_left
Numeric.shift_right_signed = op_shift_right_signed
Numeric.shift_right_unsigned = op_shift_right_unsigned

Numeric.is_greater_signed = op_is_greater_signed
Numeric.is_less_unsigned = op_is_less_unsigned
Numeric.is_greater_unsigned = op_is_greater_unsigned

Numeric.__add = op_add
Numeric.__sub = op_subtract
Numeric.__mul = op_multiply
Numeric.__div = op_divide_signed

Numeric.__unm = op_negate

Numeric.__eq = op_is_equal
Numeric.__lt = op_is_less_signed

function Numeric.__le(lhs, rhs)
	return op_is_less_signed(lhs, rhs) or op_is_equal(lhs, rhs)
end

function Numeric.__tostring(value)
	return tostring(to_f64(value))
end

return Numeric
