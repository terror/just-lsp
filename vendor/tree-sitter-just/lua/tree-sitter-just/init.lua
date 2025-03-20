local M = {}

local is_windows
if jit ~= nil then
	is_windows = jit.os == "Windows"
else
	is_windows = package.config:sub(1, 1) == "\\"
end
local get_separator = function()
	if is_windows then
		return "\\"
	end
	return "/"
end
local join_paths = function(...)
	local separator = get_separator()
	return table.concat({ ... }, separator)
end

function M.setup(arg)
	local parser_config = require("nvim-treesitter.parsers").get_parser_configs()
	parser_config.just = {
		install_info = {
			url = arg["local"] and join_paths(
				vim.fn.stdpath("data"),
				"site",
				"pack",
				"packer",
				"start",
				"tree-sitter-just"
			) or "https://github.com/IndianBoy42/tree-sitter-just", -- local path or git repo
			files = { "src/parser.c", "src/scanner.c" },
			branch = "main",
		},
		maintainers = { "@IndianBoy42" },
	}
	local ok, ft = pcall(require, "filetype")
	if ok then
		ft.setup({
			overrides = {
				extensions = {
					just = "just",
				},
				literals = {
					Justfile = "just",
					justfile = "just",
					[".Justfile"] = "just",
					[".justfile"] = "just",
				},
			},
		})
	end
end

return M
