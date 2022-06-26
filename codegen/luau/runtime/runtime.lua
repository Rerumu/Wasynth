local module = {}

local MAX_SIGNED = 0x7fffffff
local BIT_SET_32 = 0x100000000

local to_u32 = bit32.band

local num_from_u32 = I64.from_u32
local num_into_u32 = I64.into_u32

local function to_i32(num)
	if num > MAX_SIGNED then
		num = num - BIT_SET_32
	end

	return num
end

local function no_op(num)
	return num
end

do
	local temp = {}

	temp.K_ZERO = I64.K_ZERO
	temp.K_ONE = I64.K_ONE

	temp.from_u32 = num_from_u32

	module.i64 = temp
end

do
	local add = {}
	local sub = {}
	local mul = {}
	local div = {}
	local neg = {}
	local min = {}
	local max = {}
	local copysign = {}
	local nearest = {}

	local assert = assert
	local math_abs = math.abs
	local math_floor = math.floor
	local math_round = math.round
	local math_sign = math.sign
	local math_max = math.max
	local math_min = math.min

	function add.i32(a, b)
		return to_u32(a + b)
	end

	add.i64 = I64.add

	function sub.i32(a, b)
		return to_u32(a - b)
	end

	sub.i64 = I64.subtract

	function mul.i32(a, b)
		return to_u32(a * b)
	end

	mul.i64 = I64.multiply

	function div.i32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		lhs = to_i32(lhs)
		rhs = to_i32(rhs)

		return to_u32(lhs / rhs)
	end

	div.i64 = I64.divide_signed

	function div.u32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		return to_u32(lhs / rhs)
	end

	div.u64 = I64.divide_unsigned

	function neg.f32(num)
		return -num
	end

	function min.f32(a, b)
		if b == b then
			return math_min(a, b)
		else
			return b
		end
	end

	function max.f32(a, b)
		if b == b then
			return math_max(a, b)
		else
			return b
		end
	end

	function copysign.f32(lhs, rhs)
		if rhs >= 0 then
			return (math_abs(lhs))
		else
			return -math_abs(lhs)
		end
	end

	function nearest.f32(num)
		local result = math_round(num)

		if math_abs(num) % 1 == 0.5 and math_floor(math_abs(num) % 2) == 0 then
			result = result - math_sign(result)
		end

		return result
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

	local bit_and = bit32.band

	clz.i32 = bit32.countlz
	ctz.i32 = bit32.countrz

	function popcnt.i32(num)
		local count = 0

		while num ~= 0 do
			num = bit_and(num, num - 1)
			count = count + 1
		end

		return count
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

	local num_is_equal = I64.is_equal
	local num_is_greater_signed = I64.is_greater_signed
	local num_is_greater_unsigned = I64.is_greater_unsigned
	local num_is_less_signed = I64.is_less_signed
	local num_is_less_unsigned = I64.is_less_unsigned

	eq.i64 = num_is_equal

	function ne.i64(lhs, rhs)
		return not num_is_equal(lhs, rhs)
	end

	function ge.i32(lhs, rhs)
		return to_i32(lhs) >= to_i32(rhs)
	end

	function ge.i64(lhs, rhs)
		return num_is_greater_signed(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function ge.u64(lhs, rhs)
		return num_is_greater_unsigned(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function gt.i32(lhs, rhs)
		return to_i32(lhs) > to_i32(rhs)
	end

	gt.i64 = num_is_greater_signed
	gt.u64 = num_is_greater_unsigned

	function le.i32(lhs, rhs)
		return to_i32(lhs) <= to_i32(rhs)
	end

	function le.i64(lhs, rhs)
		return num_is_less_signed(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function le.u64(lhs, rhs)
		return num_is_less_unsigned(lhs, rhs) or num_is_equal(lhs, rhs)
	end

	function lt.i32(lhs, rhs)
		return to_i32(lhs) < to_i32(rhs)
	end

	lt.i64 = num_is_less_signed
	lt.u64 = num_is_less_unsigned

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

	band.i64 = I64.bit_and

	bnot.i32 = bit32.bnot
	bnot.i64 = I64.bit_not

	bor.i64 = I64.bit_or

	bxor.i64 = I64.bit_xor

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

	rotl.i32 = bit32.lrotate
	rotl.i64 = bit32.lrotate

	rotr.i32 = bit32.rrotate
	rotr.i64 = bit32.rrotate

	shl.i32 = bit32.lshift
	shl.i64 = bit32.lshift
	shl.u32 = bit32.lshift
	shl.u64 = bit32.lshift

	shr.i32 = bit32.arshift
	shr.i64 = bit32.arshift
	shr.u32 = bit32.rshift
	shr.u64 = bit32.rshift

	module.shl = shl
	module.shr = shr
	module.rotl = rotl
	module.rotr = rotr
end

do
	local wrap = {}
	local trunc = {}
	local extend = {}
	local convert = {}
	local demote = {}
	local promote = {}
	local reinterpret = {}

	local math_ceil = math.ceil
	local math_floor = math.floor

	local string_pack = string.pack
	local string_unpack = string.unpack

	local num_from_u64 = I64.from_u64
	local num_into_u64 = I64.into_u64

	local num_negate = I64.negate
	local num_is_negative = I64.is_negative

	function wrap.i32_i64(num)
		local data_1, _ = num_into_u32(num)

		return data_1
	end

	trunc.i32_f32 = to_u32
	trunc.i32_f64 = to_u32
	trunc.u32_f32 = no_op
	trunc.u32_f64 = no_op

	function trunc.i64_f32(num)
		if num < 0 then
			local temp = num_from_u64(-math_ceil(num))

			return num_negate(temp)
		else
			local temp = math_floor(num)

			return num_from_u64(temp)
		end
	end

	function trunc.i64_f64(num)
		if num < 0 then
			local temp = num_from_u64(-math_ceil(num))

			return num_negate(temp)
		else
			local temp = math_floor(num)

			return num_from_u64(temp)
		end
	end

	function trunc.f32(num)
		if num >= 0 then
			return math.floor(num)
		else
			return math.ceil(num)
		end
	end

	trunc.f64 = trunc.f32
	trunc.u64_f32 = num_from_u64
	trunc.u64_f64 = num_from_u64

	function extend.i64_i32(num)
		if num > MAX_SIGNED then
			local temp = num_from_u32(-num + BIT_SET_32, 0)

			return num_negate(temp)
		else
			return num_from_u32(num, 0)
		end
	end

	function extend.u64_i32(num)
		return num_from_u32(num, 0)
	end

	convert.f32_i32 = no_op
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

	function convert.f64_i64(num)
		if num_is_negative(num) then
			local temp = num_negate(num)

			return -num_into_u64(temp)
		else
			return num_into_u64(num)
		end
	end

	convert.f64_u64 = num_into_u64

	demote.f32_f64 = no_op

	promote.f64_f32 = no_op

	function reinterpret.i32_f32(num)
		local packed = string_pack("f", num)

		return string_unpack("<I4", packed)
	end

	function reinterpret.i64_f64(num)
		local packed = string_pack("d", num)
		local data_1, data_2 = string_unpack("<I4I4", packed)

		return num_from_u32(data_1, data_2)
	end

	function reinterpret.f32_i32(num)
		local packed = string_pack("<I4", num)

		return string_unpack("f", packed)
	end

	function reinterpret.f64_i64(num)
		local data_1, data_2 = num_into_u32(num)
		local packed = string_pack("<I4I4", data_1, data_2)

		return string_unpack("d", packed)
	end

	module.wrap = wrap
	module.trunc = trunc
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

	local bit_extract = bit32.extract
	local bit_replace = bit32.replace

	local bit_bor = bit32.bor
	local bit_band = bit32.band
	local bit_lshift = bit32.lshift
	local bit_rshift = bit32.rshift

	local math_floor = math.floor

	local string_byte = string.byte
	local string_unpack = string.unpack

	local reinterpret_f32_i32 = module.reinterpret.f32_i32
	local reinterpret_f64_i64 = module.reinterpret.f64_i64
	local reinterpret_i32_f32 = module.reinterpret.i32_f32
	local reinterpret_i64_f64 = module.reinterpret.i64_f64

	local function load_byte(data, addr)
		local value = data[math_floor(addr / 4)] or 0

		return bit_extract(value, addr % 4 * 8, 8)
	end

	local function store_byte(data, addr, value)
		local adjust = math_floor(addr / 4)

		data[adjust] = bit_replace(data[adjust] or 0, value, addr % 4 * 8, 8)
	end

	function load.i32_i8(memory, addr)
		local b = load_byte(memory.data, addr)

		if b >= 0x80 then
			return to_u32(b - 0x100)
		else
			return b
		end
	end

	function load.i32_u8(memory, addr)
		return load_byte(memory.data, addr)
	end

	function load.i32_i16(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			num = bit_band(data[addr / 4] or 0, 0xFFFF)
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)

			num = bit_bor(b1, b2)
		end

		if num >= 0x8000 then
			return to_u32(num - 0x10000)
		else
			return num
		end
	end

	function load.i32_u16(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			return bit_band(data[addr / 4] or 0, 0xFFFF)
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)

			return bit_bor(b1, b2)
		end
	end

	function load.i32(memory, addr)
		local data = memory.data

		if addr % 4 == 0 then
			-- aligned read
			return data[addr / 4] or 0
		else
			-- unaligned read
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)
			local b3 = bit_lshift(load_byte(data, addr + 2), 16)
			local b4 = bit_lshift(load_byte(data, addr + 3), 24)

			return bit_bor(b1, b2, b3, b4)
		end
	end

	function load.i64_i8(memory, addr)
		local b = load_byte(memory.data, addr)

		if b >= 0x80 then
			b = to_u32(b - 0x100)
		end

		return num_from_u32(b, 0)
	end

	function load.i64_u8(memory, addr)
		local temp = load_byte(memory.data, addr)

		return num_from_u32(temp, 0)
	end

	function load.i64_i16(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			num = bit_band(data[addr / 4] or 0, 0xFFFF)
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)

			num = bit_bor(b1, b2)
		end

		if num >= 0x8000 then
			num = to_u32(num - 0x10000)
		end

		return num_from_u32(num, 0)
	end

	function load.i64_u16(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			num = bit_band(data[addr / 4] or 0, 0xFFFF)
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)

			num = bit_bor(b1, b2)
		end

		return num_from_u32(num, 0)
	end

	function load.i64_i32(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			num = data[addr / 4] or 0
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)
			local b3 = bit_lshift(load_byte(data, addr + 2), 16)
			local b4 = bit_lshift(load_byte(data, addr + 3), 24)

			num = bit_bor(b1, b2, b3, b4)
		end

		return num_from_u32(num, 0)
	end

	function load.i64_u32(memory, addr)
		local data = memory.data
		local num

		if addr % 4 == 0 then
			num = data[addr / 4] or 0
		else
			local b1 = load_byte(data, addr)
			local b2 = bit_lshift(load_byte(data, addr + 1), 8)
			local b3 = bit_lshift(load_byte(data, addr + 2), 16)
			local b4 = bit_lshift(load_byte(data, addr + 3), 24)

			num = bit_bor(b1, b2, b3, b4)
		end

		return num_from_u32(num, 0)
	end

	local load_i32 = load.i32

	function load.i64(memory, addr)
		local data_1 = load_i32(memory, addr)
		local data_2 = load_i32(memory, addr + 4)

		return num_from_u32(data_1, data_2)
	end

	local load_i64 = load.i64

	function load.f32(memory, addr)
		local raw = load_i32(memory, addr)

		return reinterpret_f32_i32(raw)
	end

	function load.f64(memory, addr)
		local raw = load_i64(memory, addr)

		return reinterpret_f64_i64(raw)
	end

	function store.i32_n8(memory, addr, value)
		store_byte(memory.data, addr, value)
	end

	local store_i8 = store.i32_n8

	function store.i32_n16(memory, addr, value)
		store_byte(memory.data, addr, value)
		store_byte(memory.data, addr + 1, bit_rshift(value, 8))
	end

	function store.i32(memory, addr, value)
		local data = memory.data

		if addr % 4 == 0 then
			-- aligned write
			data[addr / 4] = value
		else
			-- unaligned write
			store_byte(data, addr, value)
			store_byte(data, addr + 1, bit_rshift(value, 8))
			store_byte(data, addr + 2, bit_rshift(value, 16))
			store_byte(data, addr + 3, bit_rshift(value, 24))
		end
	end

	local store_i32 = store.i32
	local store_i32_n8 = store.i32_n8
	local store_i32_n16 = store.i32_n16

	function store.i64_n8(memory, addr, value)
		local data_1, _ = num_into_u32(value)

		store_i32_n8(memory, addr, data_1)
	end

	function store.i64_n16(memory, addr, value)
		local data_1, _ = num_into_u32(value)

		store_i32_n16(memory, addr, data_1)
	end

	function store.i64_n32(memory, addr, value)
		local data_1, _ = num_into_u32(value)

		store_i32(memory, addr, data_1)
	end

	function store.i64(memory, addr, value)
		local data_1, data_2 = num_into_u32(value)

		store_i32(memory, addr, data_1)
		store_i32(memory, addr + 4, data_2)
	end

	local store_i64 = store.i64

	function store.f32(memory, addr, value)
		store_i32(memory, addr, reinterpret_i32_f32(value))
	end

	function store.f64(memory, addr, value)
		store_i64(memory, addr, reinterpret_i64_f64(value))
	end

	function store.string(memory, offset, data, len)
		len = len or #data

		local rem = len % 4

		for i = 1, len - rem, 4 do
			local v = string_unpack("<I4", data, i)

			store_i32(memory, offset + i - 1, v)
		end

		for i = len - rem + 1, len do
			local v = string_byte(data, i)

			store_i8(memory, offset + i - 1, v)
		end
	end

	function allocator.new(min, max)
		return { min = min, max = max, data = {} }
	end

	function allocator.grow(memory, num)
		local old = memory.min
		local new = old + num

		if new > memory.max then
			return -1
		else
			memory.min = new

			return old
		end
	end

	module.load = load
	module.store = store
	module.allocator = allocator
end

return module
