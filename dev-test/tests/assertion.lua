local loaded = {}
local linked = {}

local LUA_NAN_ARITHMETIC = 0 / 0
local LUA_NAN_CANONICAL = 0 / 0
local LUA_NAN_DEFAULT = 0 / 0

local function is_number_equality(lhs, rhs)
	if type(lhs) ~= "number" or type(rhs) ~= "number" then
		return false
	elseif lhs ~= lhs and rhs ~= rhs then
		return lhs ~= rhs
	end

	return math.abs(lhs - rhs) < 0.000001
end

local function assert_eq(lhs, rhs, message, level)
	if lhs == rhs or is_number_equality(lhs, rhs) then
		return
	end

	if message then
		message = ": " .. message
	else
		message = ""
	end

	lhs = tostring(lhs)
	rhs = tostring(rhs)
	level = (level or 1) + 1

	error(lhs .. " ~= " .. rhs .. message, level)
end

local function assert_neq(lhs, rhs, message, level)
	if lhs ~= rhs and not is_number_equality(lhs, rhs) then
		return
	end

	if message then
		message = ": " .. message
	else
		message = ""
	end

	lhs = tostring(lhs)
	rhs = tostring(rhs)
	level = (level or 1) + 1

	error(lhs .. " == " .. rhs .. message, level)
end

local function raw_invoke(func, ...)
	return func(...)
end

local function assert_trap(func, ...)
	if pcall(func, ...) then
		error("Failed to trap", 2)
	end
end

local function assert_return(data, wanted)
	for i, v in ipairs(wanted) do
		assert_eq(data[i], v, "Result mismatch at " .. i, 2)
	end
end

local function assert_exhaustion(func, ...)
	if pcall(func, ...) then
		error("Failed to exhaust", 2)
	end
end
