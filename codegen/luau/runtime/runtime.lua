local module = {}

local bit_and = bit32.band
local bit_xor = bit32.bxor
local bit_lshift = bit32.lshift
local bit_rshift = bit32.rshift

local num_from_u32 = Integer.from_u32
local num_into_u32 = Integer.into_u32

local function to_i32(num)
	return bit_xor(num, 0x80000000) - 0x80000000
end

local function no_op(num)
	return num
end

module.i64 = Integer

do
	local add = {}
	local sub = {}
	local mul = {}
	local div = {}
	local rem = {}
	local neg = {}
	local min = {}
	local max = {}
	local copysign = {}
	local nearest = {}

	local assert = assert

	local math_abs = math.abs
	local math_fmod = math.fmod
	local math_modf = math.modf
	local math_round = math.round
	local math_sign = math.sign
	local math_min = math.min
	local math_max = math.max

	local num_divide_signed = Integer.divide_signed
	local num_divide_unsigned = Integer.divide_unsigned

	function add.i32(lhs, rhs)
		return bit_and(lhs + rhs, 0xFFFFFFFF)
	end

	function sub.i32(lhs, rhs)
		return bit_and(lhs - rhs, 0xFFFFFFFF)
	end

	function mul.i32(lhs, rhs)
		if (lhs + rhs) < 0x8000000 then
			return bit_and(lhs * rhs, 0xFFFFFFFF)
		else
			local a16 = bit_rshift(lhs, 16)
			local a00 = bit_and(lhs, 0xFFFF)
			local b16 = bit_rshift(rhs, 16)
			local b00 = bit_and(rhs, 0xFFFF)

			local c00 = a00 * b00
			local c16 = a16 * b00 + a00 * b16

			return bit_and(c00 + bit_lshift(c16, 16), 0xFFFFFFFF)
		end
	end

	function div.i32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		lhs = to_i32(lhs)
		rhs = to_i32(rhs)

		return bit_and(math_modf(lhs / rhs), 0xFFFFFFFF)
	end

	function div.u32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		return bit_and(math_modf(lhs / rhs), 0xFFFFFFFF)
	end

	function rem.i32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		lhs = to_i32(lhs)
		rhs = to_i32(rhs)

		return bit_and(math_fmod(lhs, rhs), 0xFFFFFFFF)
	end

	add.i64 = Integer.add
	sub.i64 = Integer.subtract
	mul.i64 = Integer.multiply
	div.i64 = num_divide_signed

	function rem.i64(lhs, rhs)
		local _, remainder = num_divide_signed(lhs, rhs)

		return remainder
	end

	div.u64 = num_divide_unsigned

	function rem.u64(lhs, rhs)
		local _, remainder = num_divide_unsigned(lhs, rhs)

		return remainder
	end

	function neg.f32(num)
		return -num
	end

	function min.f32(lhs, rhs)
		if rhs == rhs then
			return math_min(lhs, rhs)
		else
			return rhs
		end
	end

	function max.f32(lhs, rhs)
		if rhs == rhs then
			return math_max(lhs, rhs)
		else
			return rhs
		end
	end

	local CP_INSTANCE = buffer.create(8)

	local buffer_write_f64 = buffer.writef64
	local buffer_read_i8 = buffer.readi8

	function copysign.f32(lhs, rhs)
		buffer_write_f64(CP_INSTANCE, 0, rhs)

		if buffer_read_i8(CP_INSTANCE, 7) >= 0 then
			return (math_abs(lhs))
		else
			return -math_abs(lhs)
		end
	end

	function nearest.f32(num)
		local result = math_round(num)

		if (math_abs(num) + 0.5) % 2 == 1 then
			return result - math_sign(result)
		else
			return result
		end
	end

	neg.f64 = neg.f32
	min.f64 = min.f32
	max.f64 = max.f32
	copysign.f64 = copysign.f32
	nearest.f64 = nearest.f32

	module.add = add
	module.sub = sub
	module.mul = mul
	module.div = div
	module.rem = rem
	module.neg = neg
	module.min = min
	module.max = max
	module.copysign = copysign
	module.nearest = nearest
end

do
	local clz = {}
	local ctz = {}
	local popcnt = {}

	local bit_countlz = bit32.countlz
	local bit_countrz = bit32.countrz

	local function popcnt_i32(num)
		local count = 0

		while num ~= 0 do
			num = bit_and(num, num - 1)
			count = count + 1
		end

		return count
	end

	popcnt.i32 = popcnt_i32

	function clz.i64(num)
		local data_1, data_2 = num_into_u32(num)
		local temp

		if data_2 == 0 then
			temp = bit_countlz(data_1) + 32
		else
			temp = bit_countlz(data_2)
		end

		return num_from_u32(temp, 0)
	end

	function ctz.i64(num)
		local data_1, data_2 = num_into_u32(num)
		local temp

		if data_1 == 0 then
			temp = bit_countrz(data_2) + 32
		else
			temp = bit_countrz(data_1)
		end

		return num_from_u32(temp, 0)
	end

	function popcnt.i64(num)
		local data_1, data_2 = num_into_u32(num)
		local temp = popcnt_i32(data_1) + popcnt_i32(data_2)

		return num_from_u32(temp, 0)
	end

	module.clz = clz
	module.ctz = ctz
	module.popcnt = popcnt
end

do
	local eq = {}
	local ne = {}
	local le = {}
	local lt = {}
	local ge = {}
	local gt = {}

	local num_is_equal = Integer.is_equal
	local num_is_less_signed = Integer.is_less_signed
	local num_is_less_unsigned = Integer.is_less_unsigned
	local num_is_greater_signed = Integer.is_greater_signed
	local num_is_greater_unsigned = Integer.is_greater_unsigned

	function le.i32(lhs, rhs)
		return to_i32(lhs) <= to_i32(rhs)
	end

	function lt.i32(lhs, rhs)
		return to_i32(lhs) < to_i32(rhs)
	end

	function ge.i32(lhs, rhs)
		return to_i32(lhs) >= to_i32(rhs)
	end

	function gt.i32(lhs, rhs)
		return to_i32(lhs) > to_i32(rhs)
	end

	eq.i64 = num_is_equal

	function ne.i64(lhs, rhs)
		return not num_is_equal(lhs, rhs)
	end

	function le.i64(lhs, rhs)
		return num_is_less_signed(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function le.u64(lhs, rhs)
		return num_is_less_unsigned(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	lt.i64 = num_is_less_signed
	lt.u64 = num_is_less_unsigned

	function ge.i64(lhs, rhs)
		return num_is_greater_signed(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function ge.u64(lhs, rhs)
		return num_is_greater_unsigned(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	gt.i64 = num_is_greater_signed
	gt.u64 = num_is_greater_unsigned

	module.eq = eq
	module.ne = ne
	module.le = le
	module.lt = lt
	module.ge = ge
	module.gt = gt
end

do
	local band = {}
	local bor = {}
	local bxor = {}
	local bnot = {}

	band.i64 = Integer.bit_and
	bor.i64 = Integer.bit_or
	bxor.i64 = Integer.bit_xor
	bnot.i64 = Integer.bit_not

	module.band = band
	module.bor = bor
	module.bxor = bxor
	module.bnot = bnot
end

do
	local shl = {}
	local shr = {}
	local rotl = {}
	local rotr = {}

	local bit_arshift = bit32.arshift
	local bit_lrotate = bit32.lrotate
	local bit_rrotate = bit32.rrotate

	function shl.i32(lhs, rhs)
		return bit_lshift(lhs, rhs % 32)
	end

	function shr.u32(lhs, rhs)
		return bit_rshift(lhs, rhs % 32)
	end

	function shr.i32(lhs, rhs)
		return bit_arshift(lhs, rhs % 32)
	end

	function rotl.i32(lhs, rhs)
		return bit_lrotate(lhs, rhs % 32)
	end

	function rotr.i32(lhs, rhs)
		return bit_rrotate(lhs, rhs % 32)
	end

	shl.i64 = Integer.shift_left
	shr.i64 = Integer.shift_right_signed
	shr.u64 = Integer.shift_right_unsigned
	rotl.i64 = Integer.rotate_left
	rotr.i64 = Integer.rotate_right

	module.shl = shl
	module.shr = shr
	module.rotl = rotl
	module.rotr = rotr
end

do
	local wrap = {}
	local truncate = {}
	local saturate = {}
	local extend = {}
	local convert = {}
	local demote = {}
	local promote = {}
	local reinterpret = {}

	local math_ceil = math.ceil
	local math_floor = math.floor
	local math_clamp = math.clamp

	local NUM_ZERO = Integer.ZERO
	local NUM_MIN_I64 = num_from_u32(0, 0x80000000)
	local NUM_MAX_I64 = num_from_u32(0xFFFFFFFF, 0x7FFFFFFF)
	local NUM_MAX_U64 = num_from_u32(0xFFFFFFFF, 0xFFFFFFFF)

	local num_from_u64 = Integer.from_u64
	local num_into_u64 = Integer.into_u64

	local num_negate = Integer.negate
	local num_is_negative = Integer.is_negative

	local function truncate_f64(num)
		if num >= 0 then
			return math_floor(num)
		else
			return math_ceil(num)
		end
	end

	function wrap.i32_i64(num)
		local data_1, _ = num_into_u32(num)

		return data_1
	end

	function truncate.i32_f32(num)
		return bit_and(truncate_f64(num), 0xFFFFFFFF)
	end

	truncate.i32_f64 = truncate.i32_f32

	truncate.u32_f32 = truncate_f64
	truncate.u32_f64 = truncate_f64

	function truncate.i64_f32(num)
		if num < 0 then
			local temp = num_from_u64(-num)

			return num_negate(temp)
		else
			return num_from_u64(num)
		end
	end

	truncate.i64_f64 = truncate.i64_f32

	function truncate.u64_f32(num)
		if num <= 0 then
			return NUM_ZERO
		else
			return num_from_u64(math_floor(num))
		end
	end

	truncate.u64_f64 = truncate.u64_f32

	truncate.f32 = truncate_f64
	truncate.f64 = truncate_f64

	function saturate.i32_f32(num)
		local temp = math_clamp(truncate_f64(num), -0x80000000, 0x7FFFFFFF)

		return bit_and(temp, 0xFFFFFFFF)
	end

	saturate.i32_f64 = saturate.i32_f32

	function saturate.u32_f32(num)
		local temp = math_clamp(truncate_f64(num), 0, 0xFFFFFFFF)

		return temp
	end

	saturate.u32_f64 = saturate.u32_f32

	local truncate_i64_f64 = truncate.i64_f64

	function saturate.i64_f32(num)
		if num >= 2 ^ 63 - 1 then
			return NUM_MAX_I64
		elseif num <= -2 ^ 63 then
			return NUM_MIN_I64
		else
			return truncate_i64_f64(num)
		end
	end

	saturate.i64_f64 = saturate.i64_f32

	function saturate.u64_f32(num)
		if num >= 2 ^ 64 then
			return NUM_MAX_U64
		elseif num <= 0 then
			return NUM_ZERO
		else
			return truncate_i64_f64(num)
		end
	end

	saturate.u64_f64 = saturate.u64_f32

	function extend.i32_n8(num)
		num = bit_and(num, 0xFF)

		if num >= 0x80 then
			return bit_and(num - 0x100, 0xFFFFFFFF)
		else
			return num
		end
	end

	function extend.i32_n16(num)
		num = bit_and(num, 0xFFFF)

		if num >= 0x8000 then
			return bit_and(num - 0x10000, 0xFFFFFFFF)
		else
			return num
		end
	end

	function extend.i64_n8(num)
		local data_1, _ = num_into_u32(num)

		data_1 = bit_and(data_1, 0xFF)

		if data_1 >= 0x80 then
			local temp = num_from_u32(-data_1 + 0x100, 0)

			return num_negate(temp)
		else
			return num_from_u32(data_1, 0)
		end
	end

	function extend.i64_n16(num)
		local data_1, _ = num_into_u32(num)

		data_1 = bit_and(data_1, 0xFFFF)

		if data_1 >= 0x8000 then
			local temp = num_from_u32(-data_1 + 0x10000, 0)

			return num_negate(temp)
		else
			return num_from_u32(data_1, 0)
		end
	end

	function extend.i64_n32(num)
		local data_1, _ = num_into_u32(num)

		if data_1 >= 0x80000000 then
			local temp = num_from_u32(-data_1 + 0x100000000, 0)

			return num_negate(temp)
		else
			return num_from_u32(data_1, 0)
		end
	end

	function extend.i64_i32(num)
		if num >= 0x80000000 then
			local temp = num_from_u32(-num + 0x100000000, 0)

			return num_negate(temp)
		else
			return num_from_u32(num, 0)
		end
	end

	function extend.i64_u32(num)
		return num_from_u32(num, 0)
	end

	convert.f32_i32 = to_i32
	convert.f32_u32 = no_op

	function convert.f32_i64(num)
		if num_is_negative(num) then
			local temp = num_negate(num)

			return -num_into_u64(temp)
		else
			return num_into_u64(num)
		end
	end

	convert.f32_u64 = num_into_u64
	convert.f64_i32 = to_i32
	convert.f64_u32 = no_op
	convert.f64_i64 = convert.f32_i64
	convert.f64_u64 = num_into_u64

	demote.f32_f64 = no_op

	promote.f64_f32 = no_op

	local RE_INSTANCE = buffer.create(8)

	local buffer_read_f32 = buffer.readf32
	local buffer_read_f64 = buffer.readf64
	local buffer_read_u32 = buffer.readu32

	local buffer_write_f32 = buffer.writef32
	local buffer_write_f64 = buffer.writef64
	local buffer_write_u32 = buffer.writeu32

	function reinterpret.i32_f32(num)
		buffer_write_f32(RE_INSTANCE, 0, num)

		return buffer_read_u32(RE_INSTANCE, 0)
	end

	function reinterpret.i64_f64(num)
		buffer_write_f64(RE_INSTANCE, 0, num)

		local data_1 = buffer_read_u32(RE_INSTANCE, 0)
		local data_2 = buffer_read_u32(RE_INSTANCE, 4)

		return num_from_u32(data_1, data_2)
	end

	function reinterpret.f32_i32(num)
		buffer_write_u32(RE_INSTANCE, 0, num)

		return buffer_read_f32(RE_INSTANCE, 0)
	end

	function reinterpret.f64_i64(num)
		local data_1, data_2 = num_into_u32(num)

		buffer_write_u32(RE_INSTANCE, 0, data_1)
		buffer_write_u32(RE_INSTANCE, 4, data_2)

		return buffer_read_f64(RE_INSTANCE, 0)
	end

	module.wrap = wrap
	module.truncate = truncate
	module.saturate = saturate
	module.extend = extend
	module.convert = convert
	module.demote = demote
	module.promote = promote
	module.reinterpret = reinterpret
end

do
	local load = {}
	local store = {}
	local allocator = {}

	local string_sub = string.sub

	local buffer_create = buffer.create
	local buffer_to_string = buffer.tostring
	local buffer_from_string = buffer.fromstring

	local buffer_len = buffer.len
	local buffer_copy = buffer.copy
	local buffer_fill = buffer.fill

	local buffer_read_i8 = buffer.readi8
	local buffer_read_u8 = buffer.readu8
	local buffer_read_i16 = buffer.readi16
	local buffer_read_u16 = buffer.readu16
	local buffer_read_i32 = buffer.readi32
	local buffer_read_u32 = buffer.readu32
	local buffer_read_f32 = buffer.readf32
	local buffer_read_f64 = buffer.readf64

	local buffer_write_u8 = buffer.writeu8
	local buffer_write_u16 = buffer.writeu16
	local buffer_write_u32 = buffer.writeu32
	local buffer_write_f32 = buffer.writef32
	local buffer_write_f64 = buffer.writef64

	function load.i32_i8(memory, addr)
		return bit_and(buffer_read_i8(memory.data, addr), 0xFFFFFFFF)
	end

	function load.i32_u8(memory, addr)
		return buffer_read_u8(memory.data, addr)
	end

	function load.i32_i16(memory, addr)
		return bit_and(buffer_read_i16(memory.data, addr), 0xFFFFFFFF)
	end

	function load.i32_u16(memory, addr)
		return buffer_read_u16(memory.data, addr)
	end

	function load.i32(memory, addr)
		return buffer_read_u32(memory.data, addr)
	end

	function load.i64_i8(memory, addr)
		local value = buffer_read_i8(memory.data, addr)

		if value >= 0 then
			return num_from_u32(value, 0)
		else
			return num_from_u32(value + 0x100000000, 0xFFFFFFFF)
		end
	end

	function load.i64_u8(memory, addr)
		return num_from_u32(buffer_read_u8(memory.data, addr), 0)
	end

	function load.i64_i16(memory, addr)
		local value = buffer_read_i16(memory.data, addr)

		if value >= 0 then
			return num_from_u32(value, 0)
		else
			return num_from_u32(value + 0x100000000, 0xFFFFFFFF)
		end
	end

	function load.i64_u16(memory, addr)
		return num_from_u32(buffer_read_u16(memory.data, addr), 0)
	end

	function load.i64_i32(memory, addr)
		local value = buffer_read_i32(memory.data, addr)

		if value >= 0 then
			return num_from_u32(value, 0)
		else
			return num_from_u32(value + 0x100000000, 0xFFFFFFFF)
		end
	end

	function load.i64_u32(memory, addr)
		return num_from_u32(buffer_read_u32(memory.data, addr), 0)
	end

	function load.i64(memory, addr)
		local data = memory.data
		local value_1 = buffer_read_u32(data, addr)
		local value_2 = buffer_read_u32(data, addr + 4)

		return num_from_u32(value_1, value_2)
	end

	function load.f32(memory, addr)
		return buffer_read_f32(memory.data, addr)
	end

	function load.f64(memory, addr)
		return buffer_read_f64(memory.data, addr)
	end

	function load.string(memory, addr, len)
		local temp = buffer_create(len)

		buffer_copy(temp, 0, memory.data, addr, len)

		return buffer_to_string(temp)
	end

	function store.i32_n8(memory, addr, value)
		buffer_write_u8(memory.data, addr, value)
	end

	function store.i32_n16(memory, addr, value)
		buffer_write_u16(memory.data, addr, value)
	end

	function store.i32(memory, addr, value)
		buffer_write_u32(memory.data, addr, value)
	end

	function store.i64_n8(memory, addr, value)
		local value_1, _ = num_into_u32(value)

		buffer_write_u8(memory.data, addr, value_1)
	end

	function store.i64_n16(memory, addr, value)
		local value_1, _ = num_into_u32(value)

		buffer_write_u16(memory.data, addr, value_1)
	end

	function store.i64_n32(memory, addr, value)
		local value_1, _ = num_into_u32(value)

		buffer_write_u32(memory.data, addr, value_1)
	end

	function store.i64(memory, addr, value)
		local data = memory.data
		local value_1, value_2 = num_into_u32(value)

		buffer_write_u32(data, addr, value_1)
		buffer_write_u32(data, addr + 4, value_2)
	end

	function store.f32(memory, addr, value)
		buffer_write_f32(memory.data, addr, value)
	end

	function store.f64(memory, addr, value)
		buffer_write_f64(memory.data, addr, value)
	end

	function store.string(memory, addr, data, len)
		local content = if not len or len == #data then data else string_sub(data, 1, len)
		local temp = buffer_from_string(content)

		buffer_copy(memory.data, addr, temp)
	end

	function store.copy(memory_1, addr_1, memory_2, addr_2, len)
		buffer_copy(memory_1.data, addr_1, memory_2.data, addr_2, len)
	end

	function store.fill(memory, addr, len, value)
		buffer_fill(memory.data, addr, value, len)
	end

	local WASM_PAGE_SIZE = 65536

	function allocator.new(min, max)
		return { max = max, data = buffer_create(min * WASM_PAGE_SIZE) }
	end

	function allocator.size(memory)
		return buffer_len(memory.data) / WASM_PAGE_SIZE
	end

	function allocator.grow(memory, num)
		local old = allocator.size(memory)
		local new = old + num

		if new <= memory.max then
			local reallocated = buffer_create(new * WASM_PAGE_SIZE)

			buffer_copy(reallocated, 0, memory.data)

			memory.data = reallocated

			return old
		else
			return 0xFFFFFFFF
		end
	end

	module.load = load
	module.store = store
	module.allocator = allocator
end

return module
