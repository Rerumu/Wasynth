do
	local WASM_PAGE_SIZE = 65536

	local function is_valid_address(memory, addr, size)
		return addr >= 0 and addr + size <= memory.min * WASM_PAGE_SIZE
	end

	local function load_checked(name, size)
		local old = assert(rt.load[name], "Missing load function " .. name)

		rt.load[name] = function(memory, addr)
			assert(is_valid_address(memory, addr, size), "Invalid memory read")

			return old(memory, addr)
		end
	end

	local function store_checked(name, size)
		local old = assert(rt.store[name], "Missing store function " .. name)

		rt.store[name] = function(memory, addr, value)
			assert(is_valid_address(memory, addr, size), "Invalid memory write")

			return old(memory, addr, value)
		end
	end

	load_checked("i32_i8", 1)
	load_checked("i32_u8", 1)
	load_checked("i32_i16", 2)
	load_checked("i32_u16", 2)
	load_checked("i32", 4)
	load_checked("i64_i8", 1)
	load_checked("i64_u8", 1)
	load_checked("i64_i16", 2)
	load_checked("i64_u16", 2)
	load_checked("i64_i32", 4)
	load_checked("i64_u32", 4)
	load_checked("i64", 8)
	load_checked("f32", 4)
	load_checked("f64", 8)

	store_checked("i32_n8", 1)
	store_checked("i32_n16", 2)
	store_checked("i32", 4)
	store_checked("i64_n8", 1)
	store_checked("i64_n16", 2)
	store_checked("i64_n32", 4)
	store_checked("i64", 8)
	store_checked("f32", 4)
	store_checked("f64", 8)
end

local loaded = {}
local linked = {}

local LUA_NAN_ARITHMETIC = -(0 / 0)
local LUA_NAN_CANONICAL = -(0 / 0)
local LUA_NAN_DEFAULT = -(0 / 0)
local LUA_INFINITY = math.huge

local function is_number_equal(lhs, rhs)
	if type(lhs) ~= "number" or type(rhs) ~= "number" then
		return false
	end

	return math.abs(lhs - rhs) < 0.00001 or string.format("%.3g", lhs) == string.format("%.3g", rhs)
end

local function assert_eq(lhs, rhs, level)
	if lhs == rhs or is_number_equal(lhs, rhs) then
		return
	end

	lhs = tostring(lhs)
	rhs = tostring(rhs)
	level = (level or 1) + 1

	error(lhs .. " ~= " .. rhs, level)
end

local function assert_neq(lhs, rhs, level)
	if lhs ~= rhs and not is_number_equal(lhs, rhs) then
		return
	end

	lhs = tostring(lhs)
	rhs = tostring(rhs)
	level = (level or 1) + 1

	error(lhs .. " == " .. rhs, level)
end

local function raw_invoke(func, ...)
	return func(...)
end

local function assert_trap(func, ...)
	if pcall(func, ...) then
		local trace = debug.traceback("Failed to trap", 2)

		io.stderr:write(trace, "\n")
	end
end

local function assert_return(data, wanted)
	for i, v in ipairs(wanted) do
		assert_eq(data[i], v, 2)
	end
end

local function assert_exhaustion(func, ...)
	if pcall(func, ...) then
		error("Failed to exhaust", 2)
	end
end

linked.spectest = {
	func_list = {
		print = print,
		print_f32 = print,
		print_f64 = print,
		print_f64_f64 = print,
		print_i32 = print,
		print_i32_f32 = print,
	},
	global_list = {
		global_f32 = { value = 666 },
		global_f64 = { value = 666 },
		global_i32 = { value = 666 },
		global_i64 = { value = 666LL },
	},
	table_list = { table = { data = {} } },
	memory_list = { memory = rt.allocator.new(1, 2) },
}
