local div = {}

local le = {}
local lt = {}
local ge = {}
local gt = {}

local band = {}
local bor = {}
local bxor = {}
local bnot = {}

local shl = {}
local shr = {}

local extend = {}
local wrap = {}

local load = {}
local store = {}

-- Helper functions
local function no_op(x) return x end

local function unsign_i32(x)
	if x < 0 then x = x + 0x100000000 end

	return x
end

local function unsign_i64(x)
	if x < 0 then x = x + 0x10000000000000000 end

	return x
end

local function rip_u64(x) return math.floor(x / 0x100000000), x % 0x100000000 end

local function merge_u64(hi, lo) return hi * 0x100000000 + lo end

-- Runtime functions
local function grow_page_num(memory, num)
	local old = memory.min
	local new = old + num

	if memory.max and new > memory.max then
		return -1
	else
		memory.min = new

		return old
	end
end

function div.i32(lhs, rhs)
	if rhs == 0 then error('division by zero') end

	return math.floor(lhs / rhs)
end

function le.u32(lhs, rhs) return unsign_i32(lhs) <= unsign_i32(rhs) and 1 or 0 end

function lt.u32(lhs, rhs) return unsign_i32(lhs) < unsign_i32(rhs) and 1 or 0 end

function ge.u32(lhs, rhs) return unsign_i32(lhs) >= unsign_i32(rhs) and 1 or 0 end

function gt.u32(lhs, rhs) return unsign_i32(lhs) > unsign_i32(rhs) and 1 or 0 end

function le.u64(lhs, rhs) return unsign_i64(lhs) <= unsign_i64(rhs) and 1 or 0 end

function lt.u64(lhs, rhs) return unsign_i64(lhs) < unsign_i64(rhs) and 1 or 0 end

function ge.u64(lhs, rhs) return unsign_i64(lhs) >= unsign_i64(rhs) and 1 or 0 end

function gt.u64(lhs, rhs) return unsign_i64(lhs) > unsign_i64(rhs) and 1 or 0 end

band.i32 = bit32.band
bor.i32 = bit32.bor
bxor.i32 = bit32.bxor
bnot.i32 = bit32.bnot
band.i64 = bit32.band
bor.i64 = bit32.bor
bxor.i64 = bit32.bxor
bnot.i64 = bit32.bnot

shl.u32 = bit32.lshift
shr.u32 = bit32.rshift
shl.i32 = bit32.lshift
shr.i32 = bit32.rshift
shl.u64 = bit32.lshift
shr.u64 = bit32.rshift
shl.i64 = bit32.lshift
shr.i64 = bit32.rshift

extend.i32_u64 = no_op

function wrap.i64_i32(i) return i % 2 ^ 32 end

function load.i32(memory, addr)
	if addr % 4 ~= 0 then error('unaligned read') end

	return memory.data[addr / 4] or 0
end

function load.i32_u8(memory, addr)
	local value = load.i32(memory, addr)

	return bit32.band(value, 0xFF)
end

function load.i64(memory, addr)
	if addr % 4 ~= 0 then error('unaligned read') end

	local hi = memory.data[addr / 4 + 1] or 0
	local lo = memory.data[addr / 4] or 0

	return merge_u64(hi, lo)
end

function store.i32(memory, addr, value)
	if addr % 4 ~= 0 then error('unaligned write') end

	memory.data[addr / 4] = value
end

function store.i32_n8(memory, addr, value)
	if addr % 4 ~= 0 then error('unaligned write') end

	local old = bit32.band(memory.data[addr / 4] or 0, 0xFFFFFF00)
	local new = bit32.band(value, 0xFF)

	memory.data[addr / 4] = bit32.bor(old, new)
end

function store.i64(memory, addr, value)
	if addr % 4 ~= 0 then error('unaligned write') end

	local hi, lo = rip_u64(value)

	memory.data[addr / 4] = lo
	memory.data[addr / 4 + 1] = hi
end

return {
	grow_page_num = grow_page_num,
	div = div,
	le = le,
	lt = lt,
	ge = ge,
	gt = gt,
	band = band,
	bor = bor,
	bxor = bxor,
	bnot = bnot,
	shl = shl,
	shr = shr,
	extend = extend,
	wrap = wrap,
	load = load,
	store = store,
}
