local module = {}

local function no_op(x) return x end

local function to_u32(x) return bit32.band(x, 0xFFFFFFFF) end

local function to_i32(x)
	if x > 0x7FFFFFFF then x = x - 0x100000000 end

	return x
end

local function wrap_i32(x) return to_i32(to_u32(x) % 0x100000000) end

local function truncate(num)
	if num >= 0 then
		return math.floor(num)
	else
		return math.ceil(num)
	end
end

do
	local add = {}
	local sub = {}
	local mul = {}
	local div = {}

	function add.i32(a, b) return wrap_i32(a + b) end

	function sub.i32(a, b) return wrap_i32(a - b) end

	function mul.i32(a, b) return wrap_i32(a * b) end

	function div.i32(lhs, rhs)
		assert(rhs ~= 0, 'division by zero')

		return truncate(lhs / rhs)
	end

	function div.u32(lhs, rhs)
		assert(rhs ~= 0, 'division by zero')

		lhs = to_u32(lhs)
		rhs = to_u32(rhs)

		return to_i32(math.floor(lhs / rhs))
	end

	module.add = add
	module.sub = sub
	module.mul = mul
	module.div = div
end

do
	local clz = {}
	local ctz = {}
	local popcnt = {}

	clz.i32 = bit32.countlz
	ctz.i32 = bit32.countrz

	function popcnt.i32(num)
		local count = 0

		while num ~= 0 do
			num = bit32.band(num, num - 1)
			count = count + 1
		end

		return count
	end

	module.clz = clz
	module.ctz = ctz
	module.popcnt = popcnt
end

do
	local eqz = {}
	local eq = {}
	local ne = {}
	local le = {}
	local lt = {}
	local ge = {}
	local gt = {}

	local function to_boolean(cond)
		if cond then
			return 1
		else
			return 0
		end
	end

	function eq.i32(lhs, rhs) return to_boolean(lhs == rhs) end
	function eq.num(lhs, rhs) return to_boolean(lhs == rhs) end

	function eqz.i32(lhs) return to_boolean(lhs == 0) end

	function ne.i32(lhs, rhs) return to_boolean(lhs ~= rhs) end
	function ne.num(lhs, rhs) return to_boolean(lhs ~= rhs) end

	function ge.i32(lhs, rhs) return to_boolean(lhs >= rhs) end
	function ge.u32(lhs, rhs) return to_boolean(to_u32(lhs) >= to_u32(rhs)) end

	function gt.i32(lhs, rhs) return to_boolean(lhs > rhs) end
	function gt.u32(lhs, rhs) return to_boolean(to_u32(lhs) > to_u32(rhs)) end

	function le.i32(lhs, rhs) return to_boolean(lhs <= rhs) end
	function le.u32(lhs, rhs) return to_boolean(to_u32(lhs) <= to_u32(rhs)) end

	function lt.i32(lhs, rhs) return to_boolean(lhs < rhs) end
	function lt.u32(lhs, rhs) return to_boolean(to_u32(lhs) < to_u32(rhs)) end

	module.eqz = eqz
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

	band.i32 = bit32.band

	bnot.i32 = bit32.bnot

	bor.i32 = bit32.bor

	bxor.i32 = bit32.bxor

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

	rotr.i32 = bit32.rrotate

	shl.i32 = bit32.lshift
	shl.u32 = bit32.lshift

	shr.i32 = bit32.arshift
	shr.u32 = bit32.rshift

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

	trunc.i32_f32 = truncate
	trunc.i32_f64 = truncate
	trunc.u32_f32 = truncate
	trunc.u32_f64 = truncate

	extend.i64_i32 = no_op

	function convert.f32_i32(num) return num end

	function convert.f64_i32(num) return num end

	module.wrap = wrap
	module.trunc = trunc
	module.extend = extend
	module.convert = convert
	module.reinterpret = reinterpret
end

do
	local load = {}
	local store = {}
	local memory = {}

	local function rip_u64(x) return math.floor(x / 0x100000000), x % 0x100000000 end

	local function merge_u64(hi, lo) return hi * 0x100000000 + lo end

	local function black_mask_byte(value, offset)
		local mask = bit32.lshift(0xFF, offset * 8)

		return bit32.band(value, bit32.bnot(mask))
	end

	local function load_byte(memory, addr)
		local offset = addr % 4
		local value = memory.data[(addr - offset) / 4] or 0

		return bit32.band(bit32.rshift(value, offset * 8), 0xFF)
	end

	local function store_byte(memory, addr, value)
		local offset = addr % 4
		local adjust = (addr - offset) / 4
		local lhs = bit32.lshift(bit32.band(value, 0xFF), offset * 8)
		local rhs = black_mask_byte(memory.data[adjust] or 0, offset)

		memory.data[adjust] = bit32.bor(lhs, rhs)
	end

	function load.i32_i8(memory, addr)
		local b = load_byte(memory, addr)

		if b > 0x7F then b = b - 0x100 end

		return b
	end

	load.i32_u8 = load_byte

	function load.i32(memory, addr)
		if addr % 4 == 0 then
			-- aligned read
			return memory.data[addr / 4] or 0
		else
			-- unaligned read
			local b1 = load_byte(memory, addr)
			local b2 = bit32.lshift(load_byte(memory, addr + 1), 8)
			local b3 = bit32.lshift(load_byte(memory, addr + 2), 16)
			local b4 = bit32.lshift(load_byte(memory, addr + 3), 24)

			return bit32.bor(b1, b2, b3, b4)
		end
	end

	function load.i64(memory, addr)
		local hi = load.i32(memory, addr + 4)
		local lo = load.i32(memory, addr)

		return merge_u64(hi, lo)
	end

	store.i32_n8 = store_byte

	function store.i32(memory, addr, value)
		if addr % 4 == 0 then
			-- aligned write
			memory.data[addr / 4] = value
		else
			-- unaligned write
			store_byte(memory, addr, value)
			store_byte(memory, addr + 1, bit32.rshift(value, 8))
			store_byte(memory, addr + 2, bit32.rshift(value, 16))
			store_byte(memory, addr + 3, bit32.rshift(value, 24))
		end
	end

	function store.i64(memory, addr, value)
		local hi, lo = rip_u64(value)

		store.i32(memory, addr, lo)
		store.i32(memory, addr + 4, hi)
	end

	function memory.new(min, max) return {min = min, max = max, data = {}} end

	function memory.init(memory, offset, data)
		local store_i8 = module.store.i32_n8
		local store_i32 = module.store.i32

		local len = #data
		local rem = len % 4

		for i = 1, len - rem, 4 do
			local v = string.unpack('<I4', data, i)

			store_i32(memory, offset + i - 1, v)
		end

		for i = len - rem + 1, len do
			local v = string.byte(data, i)

			store_i8(memory, offset + i - 1, v)
		end
	end

	function memory.grow(memory, num)
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
	module.memory = memory
end

return module
