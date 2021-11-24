local WASM_PAGE_SIZE = 65536

local bit = require('bit')
local ffi = require('ffi')

local module = {}

local vla_u8 = ffi.typeof('uint8_t[?]')

local ptr_i8 = ffi.typeof('int8_t *')
local ptr_i16 = ffi.typeof('int16_t *')
local ptr_i32 = ffi.typeof('int32_t *')
local ptr_i64 = ffi.typeof('int64_t *')

local ptr_u16 = ffi.typeof('uint16_t *')
local ptr_u32 = ffi.typeof('uint32_t *')

local ptr_f32 = ffi.typeof('float *')
local ptr_f64 = ffi.typeof('double *')

local u32 = ffi.typeof('uint32_t')
local u64 = ffi.typeof('uint64_t')
local i32 = ffi.typeof('int32_t')
local i64 = ffi.typeof('int64_t')

ffi.cdef [[
typedef union {
	int32_t i32;
	int64_t i64;
	float f32;
	double f64;
} Reinterpret;
]]

do
	local div = {}

	module.div = div

	function div.i32(lhs, rhs)
		if rhs == 0 then error('division by zero') end

		return math.floor(lhs / rhs)
	end

	function div.u32(lhs, rhs)
		if rhs == 0 then error('division by zero') end

		lhs = tonumber(u32(lhs))
		rhs = tonumber(u32(rhs))

		return math.floor(lhs / rhs)
	end

	function div.u64(lhs, rhs)
		if rhs == 0 then error('division by zero') end

		return i64(u64(lhs) / u64(rhs))
	end
end

do
	local clz = {}
	local ctz = {}
	local popcnt = {}

	module.clz = clz
	module.ctz = ctz
	module.popcnt = popcnt

	function clz.i32(num)
		for i = 0, 31 do
			local mask = bit.lshift(1, 31 - i)

			if bit.band(num, mask) ~= 0 then return i end
		end

		return 32
	end

	function ctz.i32(num)
		for i = 0, 31 do
			local mask = bit.lshift(1, i)

			if bit.band(num, mask) ~= 0 then return i end
		end

		return 32
	end

	function popcnt.i32(num)
		local count = 0

		while num ~= 0 do
			num = bit.band(num, num - 1)
			count = count + 1
		end

		return count
	end
end

do
	local eqz = {}
	local eq = {}
	local ne = {}
	local le = {}
	local lt = {}
	local ge = {}
	local gt = {}

	module.eqz = eqz
	module.eq = eq
	module.ne = ne
	module.le = le
	module.lt = lt
	module.ge = ge
	module.gt = gt

	local function to_boolean(cond)
		if cond then
			return 1
		else
			return 0
		end
	end

	function eq.i32(lhs, rhs) return to_boolean(lhs == rhs) end
	function eq.i64(lhs, rhs) return to_boolean(lhs == rhs) end
	function eqz.i32(lhs) return to_boolean(lhs == 0) end
	function eqz.i64(lhs) return to_boolean(lhs == 0) end
	function ne.i32(lhs, rhs) return to_boolean(lhs ~= rhs) end
	function ne.i64(lhs, rhs) return to_boolean(lhs ~= rhs) end

	function ge.i32(lhs, rhs) return to_boolean(lhs >= rhs) end
	function ge.i64(lhs, rhs) return to_boolean(lhs >= rhs) end
	function ge.u32(lhs, rhs) return to_boolean(u32(lhs) >= u32(rhs)) end
	function ge.u64(lhs, rhs) return to_boolean(u64(lhs) >= u64(rhs)) end

	function gt.i32(lhs, rhs) return to_boolean(lhs > rhs) end
	function gt.i64(lhs, rhs) return to_boolean(lhs > rhs) end
	function gt.u32(lhs, rhs) return to_boolean(u32(lhs) > u32(rhs)) end
	function gt.u64(lhs, rhs) return to_boolean(u64(lhs) > u64(rhs)) end

	function le.i32(lhs, rhs) return to_boolean(lhs <= rhs) end
	function le.i64(lhs, rhs) return to_boolean(lhs <= rhs) end
	function le.u32(lhs, rhs) return to_boolean(u32(lhs) <= u32(rhs)) end
	function le.u64(lhs, rhs) return to_boolean(u64(lhs) <= u64(rhs)) end

	function lt.i32(lhs, rhs) return to_boolean(lhs < rhs) end
	function lt.i64(lhs, rhs) return to_boolean(lhs < rhs) end
	function lt.u32(lhs, rhs) return to_boolean(u32(lhs) < u32(rhs)) end
	function lt.u64(lhs, rhs) return to_boolean(u64(lhs) < u64(rhs)) end
end

do
	local band = {}
	local bor = {}
	local bxor = {}
	local bnot = {}

	module.band = band
	module.bor = bor
	module.bxor = bxor
	module.bnot = bnot

	band.i32 = bit.band
	band.i64 = bit.band
	bnot.i32 = bit.bnot
	bnot.i64 = bit.bnot
	bor.i32 = bit.bor
	bor.i64 = bit.bor
	bxor.i32 = bit.bxor
	bxor.i64 = bit.bxor
end

do
	local shl = {}
	local shr = {}
	local rotl = {}
	local rotr = {}

	module.shl = shl
	module.shr = shr
	module.rotl = rotl
	module.rotr = rotr

	function shr.u32(lhs, rhs)
		local v = bit.rshift(u32(lhs), rhs)

		return tonumber(i32(v))
	end

	function shr.u64(lhs, rhs)
		local v = bit.rshift(u64(lhs), rhs)

		return i64(v)
	end

	rotl.i32 = bit.rol
	rotl.i64 = bit.rol
	rotr.i32 = bit.ror
	rotr.i64 = bit.ror

	shl.i32 = bit.lshift
	shl.i64 = bit.lshift
	shl.u32 = bit.lshift
	shl.u64 = bit.lshift
	shr.i32 = bit.rshift
	shr.i64 = bit.rshift
end

do
	local extend = {}
	local wrap = {}
	local convert = {}
	local reinterpret = {}

	-- This would surely be an issue in a multi-thread environment...
	-- ... thankfully this isn't one.
	local RE_INSTANCE = ffi.new('Reinterpret')

	module.extend = extend
	module.wrap = wrap
	module.convert = convert
	module.reinterpret = reinterpret

	function extend.u64_i32(num) return i64(u64(num)) end

	function wrap.i32_i64(num) return tonumber(i32(num)) end

	function convert.f32_i32(num) return num end
	function convert.f32_u32(num) return tonumber(u32(num)) end
	function convert.f32_i64(num) return tonumber(num) end
	function convert.f32_u64(num) return tonumber(u64(num)) end

	function convert.f64_i32(num) return num end
	function convert.f64_u32(num) return tonumber(u32(num)) end
	function convert.f64_i64(num) return tonumber(num) end
	function convert.f64_u64(num) return tonumber(u64(num)) end

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
end

do
	local load = {}
	local store = {}

	module.load = load
	module.store = store

	function load.i32_i8(memory, addr) return ffi.cast(ptr_i8, memory.data)[addr] end
	function load.i32_u8(memory, addr) return memory.data[addr] end
	function load.i32_i16(memory, addr) return ffi.cast(ptr_i16, memory.data + addr)[0] end
	function load.i32_u16(memory, addr) return ffi.cast(ptr_u16, memory.data + addr)[0] end
	function load.i32(memory, addr) return ffi.cast(ptr_i32, memory.data + addr)[0] end

	function load.i64_i8(memory, addr) return i64(ffi.cast(ptr_i8, memory.data)[addr]) end
	function load.i64_u8(memory, addr) return i64(memory.data[addr]) end
	function load.i64_i16(memory, addr) return i64(ffi.cast(ptr_i16, memory.data + addr)[0]) end
	function load.i64_u16(memory, addr) return i64(ffi.cast(ptr_u16, memory.data + addr)[0]) end
	function load.i64_i32(memory, addr) return i64(ffi.cast(ptr_i32, memory.data + addr)[0]) end
	function load.i64_u32(memory, addr) return i64(ffi.cast(ptr_u32, memory.data + addr)[0]) end
	function load.i64(memory, addr) return i64(ffi.cast(ptr_i64, memory.data + addr)[0]) end

	function load.f32(memory, addr) return ffi.cast(ptr_f32, memory.data + addr)[0] end
	function load.f64(memory, addr) return ffi.cast(ptr_f64, memory.data + addr)[0] end

	function store.i32_n8(memory, addr, value) memory.data[addr] = value end
	function store.i32_n16(memory, addr, value) ffi.cast(ptr_i16, memory.data + addr)[0] = value end
	function store.i32(memory, addr, value) ffi.cast(ptr_i32, memory.data + addr)[0] = value end

	function store.i64_n8(memory, addr, value) memory.data[addr] = value end
	function store.i64_n16(memory, addr, value) ffi.cast(ptr_i16, memory.data + addr)[0] = value end
	function store.i64_n32(memory, addr, value) ffi.cast(ptr_i32, memory.data + addr)[0] = value end
	function store.i64(memory, addr, value) ffi.cast(ptr_i64, memory.data + addr)[0] = value end

	function store.f32(memory, addr, value) ffi.cast(ptr_f32, memory.data + addr)[0] = value end
	function store.f64(memory, addr, value) ffi.cast(ptr_f64, memory.data + addr)[0] = value end
end

do
	local memory = {}

	module.memory = memory

	local function grow_unchecked(memory, old, new)
		local data = vla_u8(new * WASM_PAGE_SIZE, 0)

		ffi.copy(data, memory.data, old * WASM_PAGE_SIZE)

		memory.min = new
		memory.data = data
	end

	function memory.new(min, max)
		local memory = {}

		memory.min = min
		memory.max = max
		memory.data = ffi.new(vla_u8, min * WASM_PAGE_SIZE)

		return memory
	end

	function memory.init(memory, offset, data) ffi.copy(memory.data + offset, data) end

	function memory.size(memory) return memory.min end

	function memory.grow(memory, num)
		local old = memory.min
		local new = old + num

		if memory.max and new > memory.max then
			return -1
		else
			grow_unchecked(memory, old, new)

			return old
		end
	end
end

return module
