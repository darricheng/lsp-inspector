vim.lsp.config["biome"] = {
	-- Command and arguments to start the server.
	cmd = { "../target/debug/lsp-inspector" },
	-- Filetypes to automatically attach to.
	filetypes = { "typescript", "typescript.tsx", "typescriptreact" },
	-- Sets the "workspace" to the directory where any of these files is found.
	-- Files that share a root directory will reuse the LSP server connection.
	-- Nested lists indicate equal priority, see |vim.lsp.Config|.
	root_markers = { "biome.json" },
	-- Specific settings to send to the server. The schema is server-defined.
	-- settings = {},
}

vim.lsp.enable("biome")
