local module = {}

local bit = require("bit")
local ffi = require("ffi")

local u32 = ffi.typeof("uint32_t")
local u64 = ffi.typeof("uint64_t")
local i64 = ffi.typeof("int64_t")

local math_ceil = math.ceil
local math_floor = math.floor
local to_number = tonumber

local ID_ZERO = i64(0)
local ID_ONE = i64(1)

local function truncate(num)
	if num >= 0 then
		return (math_floor(num))
	else
		return (math_ceil(num))
	end
end

do
	local add = {}
	local sub = {}
	local mul = {}
	local div = {}
	local rem = {}
	local neg = {}
	local copysign = {}
	local nearest = {}

	local to_signed = bit.tobit
	local math_abs = math.abs

	local RE_INSTANCE = ffi.new([[union {
		double f64;
		struct { int32_t a32, b32; };
	}]])

	local function round(num)
		if num >= 0 then
			return (math_floor(num + 0.5))
		else
			return (math_ceil(num - 0.5))
		end
	end

	function add.i32(a, b)
		return (to_signed(a + b))
	end

	function sub.i32(a, b)
		return (to_signed(a - b))
	end

	function mul.i32(a, b)
		return (to_signed(ID_ONE * a * b))
	end

	function div.i32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		return (truncate(lhs / rhs))
	end

	function div.u32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		lhs = to_number(u32(lhs))
		rhs = to_number(u32(rhs))

		return (to_signed(math_floor(lhs / rhs)))
	end

	function rem.u32(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		lhs = to_number(u32(lhs))
		rhs = to_number(u32(rhs))

		return (to_signed(lhs % rhs))
	end

	function div.u64(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		return (i64(u64(lhs) / u64(rhs)))
	end

	function rem.u64(lhs, rhs)
		assert(rhs ~= 0, "division by zero")

		return (i64(u64(lhs) % u64(rhs)))
	end

	function neg.f32(num)
		return -num
	end

	function copysign.f32(lhs, rhs)
		RE_INSTANCE.f64 = rhs

		if RE_INSTANCE.b32 >= 0 then
			return (math_abs(lhs))
		else
			return -math_abs(lhs)
		end
	end

	function nearest.f32(num)
		local result = round(num)

		if math_abs(num) % 1 == 0.5 and result % 2 == 1 then
			result = result - 1
		end

		return result
	end

	neg.f64 = neg.f32
	copysign.f64 = copysign.f32
	nearest.f64 = nearest.f32

	module.add = add
	module.sub = sub
	module.mul = mul
	module.div = div
	module.rem = rem
	module.neg = neg
	module.copysign = copysign
	module.nearest = nearest
end

do
	local clz = {}
	local ctz = {}
	local popcnt = {}

	local lj_band = bit.band
	local lj_lshift = bit.lshift

	function clz.i32(num)
		for i = 0, 31 do
			local mask = lj_lshift(1, 31 - i)

			if lj_band(num, mask) ~= 0 then
				return i
			end
		end

		return 32
	end

	function ctz.i32(num)
		for i = 0, 31 do
			local mask = lj_lshift(1, i)

			if lj_band(num, mask) ~= 0 then
				return i
			end
		end

		return 32
	end

	function popcnt.i32(num)
		local count = 0

		while num ~= 0 do
			num = lj_band(num, num - 1)
			count = count + 1
		end

		return count
	end

	function clz.i64(num)
		for i = 0, 63 do
			local mask = lj_lshift(ID_ONE, 63 - i)

			if lj_band(num, mask) ~= ID_ZERO then
				return i * ID_ONE
			end
		end

		return 64 * ID_ONE
	end

	function ctz.i64(num)
		for i = 0, 63 do
			local mask = lj_lshift(ID_ONE, i)

			if lj_band(num, mask) ~= ID_ZERO then
				return i * ID_ONE
			end
		end

		return 64 * ID_ONE
	end

	function popcnt.i64(num)
		local count = ID_ZERO

		while num ~= ID_ZERO do
			num = lj_band(num, num - 1)
			count = count + ID_ONE
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

	function le.u32(lhs, rhs)
		return u32(lhs) <= u32(rhs)
	end

	function lt.u32(lhs, rhs)
		return u32(lhs) < u32(rhs)
	end

	function ge.u32(lhs, rhs)
		return u32(lhs) >= u32(rhs)
	end

	function gt.u32(lhs, rhs)
		return u32(lhs) > u32(rhs)
	end

	function le.u64(lhs, rhs)
		return u64(lhs) <= u64(rhs)
	end

	function lt.u64(lhs, rhs)
		return u64(lhs) < u64(rhs)
	end

	function ge.u64(lhs, rhs)
		return u64(lhs) >= u64(rhs)
	end

	function gt.u64(lhs, rhs)
		return u64(lhs) > u64(rhs)
	end

	module.le = le
	module.lt = lt
	module.ge = ge
	module.gt = gt
end

do
	local wrap = {}
	local trunc = {}
	local extend = {}
	local convert = {}
	local promote = {}
	local demote = {}
	local reinterpret = {}

	local bit_band = bit.band

	-- This would surely be an issue in a multi-thread environment...
	-- ... thankfully this isn't one.
	local RE_INSTANCE = ffi.new([[union {
		int32_t i32;
		int64_t i64;
		float f32;
		double f64;
	}]])

	function wrap.i32_i64(num)
		RE_INSTANCE.i64 = num

		return RE_INSTANCE.i32
	end

	trunc.i32_f32 = truncate
	trunc.i32_f64 = truncate
	trunc.u32_f32 = math_floor
	trunc.u32_f64 = math_floor
	trunc.i64_f32 = i64
	trunc.i64_f64 = i64
	trunc.u64_f32 = i64
	trunc.u64_f64 = i64

	function extend.i32_i8(num)
		num = bit_band(num, 0xFF)

		if num >= 0x80 then
			return num - 0x100
		else
			return num
		end
	end

	function extend.i32_i16(num)
		num = bit_band(num, 0xFFFF)

		if num >= 0x8000 then
			return num - 0x10000
		else
			return num
		end
	end

	function extend.i64_i8(num)
		num = bit_band(num, 0xFF)

		if num >= 0x80 then
			return num - 0x100
		else
			return num
		end
	end

	function extend.i64_i16(num)
		num = bit_band(num, 0xFFFF)

		if num >= 0x8000 then
			return num - 0x10000
		else
			return num
		end
	end

	function extend.i64_i32(num)
		num = bit_band(num, 0xFFFFFFFF)

		if num >= 0x80000000 then
			return num - 0x100000000
		else
			return num
		end
	end

	function extend.u64_i32(num)
		RE_INSTANCE.i64 = ID_ZERO
		RE_INSTANCE.i32 = num

		return RE_INSTANCE.i64
	end

	function convert.f32_i32(num)
		return num
	end

	function convert.f32_u32(num)
		return (to_number(u32(num)))
	end

	function convert.f32_i64(num)
		return (to_number(num))
	end

	function convert.f32_u64(num)
		return (to_number(u64(num)))
	end

	function convert.f64_i32(num)
		return num
	end

	function convert.f64_u32(num)
		return (to_number(u32(num)))
	end

	function convert.f64_i64(num)
		return (to_number(num))
	end

	function convert.f64_u64(num)
		return (to_number(u64(num)))
	end

	function demote.f32_f64(num)
		return num
	end

	function promote.f64_f32(num)
		return num
	end

	function reinterpret.i32_f32(num)
		RE_INSTANCE.f32 = num

		return RE_INSTANCE.i32
	end

	function reinterpret.i64_f64(num)
		RE_INSTANCE.f64 = num

		return RE_INSTANCE.i64
	end

	function reinterpret.f32_i32(num)
		RE_INSTANCE.i32 = num

		return RE_INSTANCE.f32
	end

	function reinterpret.f64_i64(num)
		RE_INSTANCE.i64 = num

		return RE_INSTANCE.f64
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

	ffi.cdef([[
	union Any {
		int8_t i8;
		int16_t i16;
		int32_t i32;
		int64_t i64;

		uint8_t u8;
		uint16_t u16;
		uint32_t u32;
		uint64_t u64;

		float f32;
		double f64;
	};

	struct Memory {
		uint32_t min;
		uint32_t max;
		union Any *data;
	};

	void *calloc(size_t num, size_t size);
	void *realloc(void *ptr, size_t size);
	void free(void *ptr);
	]])

	local alias_t = ffi.typeof("uint8_t *")
	local any_t = ffi.typeof("union Any *")
	local cast = ffi.cast

	local function by_offset(pointer, offset)
		local aliased = cast(alias_t, pointer)

		return cast(any_t, aliased + offset)
	end

	function load.i32_i8(memory, addr)
		return by_offset(memory.data, addr).i8
	end

	function load.i32_u8(memory, addr)
		return by_offset(memory.data, addr).u8
	end

	function load.i32_i16(memory, addr)
		return by_offset(memory.data, addr).i16
	end

	function load.i32_u16(memory, addr)
		return by_offset(memory.data, addr).u16
	end

	function load.i32(memory, addr)
		return by_offset(memory.data, addr).i32
	end

	function load.i64_i8(memory, addr)
		return (i64(by_offset(memory.data, addr).i8))
	end

	function load.i64_u8(memory, addr)
		return (i64(by_offset(memory.data, addr).u8))
	end

	function load.i64_i16(memory, addr)
		return (i64(by_offset(memory.data, addr).i16))
	end

	function load.i64_u16(memory, addr)
		return (i64(by_offset(memory.data, addr).u16))
	end

	function load.i64_i32(memory, addr)
		return (i64(by_offset(memory.data, addr).i32))
	end

	function load.i64_u32(memory, addr)
		return (i64(by_offset(memory.data, addr).u32))
	end

	function load.i64(memory, addr)
		return by_offset(memory.data, addr).i64
	end

	function load.f32(memory, addr)
		return by_offset(memory.data, addr).f32
	end

	function load.f64(memory, addr)
		return by_offset(memory.data, addr).f64
	end

	function store.i32_n8(memory, addr, value)
		by_offset(memory.data, addr).i8 = value
	end

	function store.i32_n16(memory, addr, value)
		by_offset(memory.data, addr).i16 = value
	end

	function store.i32(memory, addr, value)
		by_offset(memory.data, addr).i32 = value
	end

	function store.i64_n8(memory, addr, value)
		by_offset(memory.data, addr).i8 = value
	end

	function store.i64_n16(memory, addr, value)
		by_offset(memory.data, addr).i16 = value
	end

	function store.i64_n32(memory, addr, value)
		by_offset(memory.data, addr).i32 = value
	end

	function store.i64(memory, addr, value)
		by_offset(memory.data, addr).i64 = value
	end

	function store.f32(memory, addr, value)
		by_offset(memory.data, addr).f32 = value
	end

	function store.f64(memory, addr, value)
		by_offset(memory.data, addr).f64 = value
	end

	function store.string(memory, addr, data, len)
		local start = by_offset(memory.data, addr)

		ffi.copy(start, data, len or #data)
	end

	local WASM_PAGE_SIZE = 65536

	local function finalizer(memory)
		ffi.C.free(memory.data)
	end

	local function grow_unchecked(memory, old, new)
		memory.data = ffi.C.realloc(memory.data, new)

		assert(memory.data ~= nil, "failed to reallocate")

		ffi.fill(by_offset(memory.data, old), new - old, 0)
	end

	function allocator.new(min, max)
		local data = ffi.C.calloc(max, WASM_PAGE_SIZE)

		assert(data ~= nil, "failed to allocate")

		local memory = ffi.new("struct Memory", min, max, data)

		return ffi.gc(memory, finalizer)
	end

	function allocator.grow(memory, num)
		if num == 0 then
			return memory.min
		end

		local old = memory.min
		local new = old + num

		if new > memory.max then
			return -1
		else
			grow_unchecked(memory, old * WASM_PAGE_SIZE, new * WASM_PAGE_SIZE)
			memory.min = new

			return old
		end
	end

	module.load = load
	module.store = store
	module.allocator = allocator
end

return module
