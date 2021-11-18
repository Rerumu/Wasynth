local WASM_PAGE_SIZE = 65536

local bit = require('bit')
local ffi = require('ffi')

local module = {}

local vla_u8 = ffi.typeof('uint8_t[?]')

local ptr_i8 = ffi.typeof('int8_t *')
local ptr_i64 = ffi.typeof('int64_t *')
local ptr_i32 = ffi.typeof('int32_t *')

local u32 = ffi.typeof('uint32_t')
local u64 = ffi.typeof('uint64_t')
local i32 = ffi.typeof('int32_t')
local i64 = ffi.typeof('int64_t')

do
	local div = {}

	module.div = div

	function div.i32(lhs, rhs)
		if rhs == 0 then error('division by zero') end

		return math.floor(lhs / rhs)
	end
end

do
	local le = {}
	local lt = {}
	local ge = {}
	local gt = {}

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

	function ge.u32(lhs, rhs) return to_boolean(u32(lhs) >= u32(rhs)) end
	function ge.u64(lhs, rhs) return to_boolean(u64(lhs) >= u64(rhs)) end
	function gt.u32(lhs, rhs) return to_boolean(u32(lhs) > u32(rhs)) end
	function gt.u64(lhs, rhs) return to_boolean(u64(lhs) > u64(rhs)) end
	function le.u32(lhs, rhs) return to_boolean(u32(lhs) <= u32(rhs)) end
	function le.u64(lhs, rhs) return to_boolean(u64(lhs) <= u64(rhs)) end
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

	module.shl = shl
	module.shr = shr

	function shr.u32(lhs, rhs)
		local v = bit.rshift(u32(lhs), rhs)

		return tonumber(i32(v))
	end

	function shr.u64(lhs, rhs)
		local v = bit.rshift(u64(lhs), rhs)

		return i64(v)
	end

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

	module.extend = extend
	module.wrap = wrap

	extend.i32_u64 = i64
	wrap.i64_i32 = i32
end

do
	local load = {}
	local store = {}

	module.load = load
	module.store = store

	function load.i32_i8(memory, addr) return ffi.cast(ptr_i8, memory.data)[addr] end
	function load.i32_u8(memory, addr) return memory.data[addr] end
	function load.i32(memory, addr) return ffi.cast(ptr_i32, memory.data + addr)[0] end
	function load.i64(memory, addr) return ffi.cast(ptr_i64, memory.data + addr)[0] end
	function store.i32_n8(memory, addr, value) memory.data[addr] = value end
	function store.i32(memory, addr, value) ffi.cast(ptr_i32, memory.data + addr)[0] = value end
	function store.i64(memory, addr, value) ffi.cast(ptr_i64, memory.data + addr)[0] = value end
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
