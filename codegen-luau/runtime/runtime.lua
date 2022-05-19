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
	local add = {}
	local sub = {}
	local mul = {}
	local div = {}

	local assert = assert

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

	module.add = add
	module.sub = sub
	module.mul = mul
	module.div = div
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
	local le = {}
	local lt = {}
	local ge = {}
	local gt = {}

	local num_is_equal = I64.is_equal
	local num_is_greater_signed = I64.is_greater_signed
	local num_is_greater_unsigned = I64.is_greater_unsigned
	local num_is_less_signed = I64.is_less_signed
	local num_is_less_unsigned = I64.is_less_unsigned

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

	band.i32 = bit32.band
	band.i64 = I64.bit_and

	bnot.i32 = bit32.bnot
	bnot.i64 = I64.bit_not

	bor.i32 = bit32.bor
	bor.i64 = I64.bit_or

	bxor.i32 = bit32.bxor
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
	module.reinterpret = reinterpret
end

do
	local load = {}
	local store = {}
	local allocator = {}

	local bit_extract = bit32.extract
	local bit_replace = bit32.replace

	local bit_bor = bit32.bor
	local bit_lshift = bit32.lshift
	local bit_rshift = bit32.rshift

	local math_floor = math.floor

	local string_byte = string.byte
	local string_unpack = string.unpack

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

		if b > 0x7F then
			b = b - 0x100
		end

		return b
	end

	function load.i32_u8(memory, addr)
		return load_byte(memory.data, addr)
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

	local load_i32 = load.i32

	function load.i64(memory, addr)
		local data_1 = load_i32(memory, addr)
		local data_2 = load_i32(memory, addr + 4)

		return num_from_u32(data_1, data_2)
	end

	function store.i32_i8(memory, addr, value)
		store_byte(memory.data, addr, value)
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

	function store.i64(memory, addr, value)
		local data_1, data_2 = num_into_u32(value)

		store_i32(memory, addr, data_1)
		store_i32(memory, addr + 4, data_2)
	end

	function allocator.new(min, max)
		return { min = min, max = max, data = {} }
	end

	local store_i8 = store.i32_n8

	function allocator.init(memory, offset, data)
		local len = #data
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
