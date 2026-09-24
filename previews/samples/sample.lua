-- Apply the colourscheme and report what resolved.
local M = {}

local ROLES = { "base", "surface0", "text", "red", "aqua" }

--- Look a colour up by role, falling back to the default.
---@param role string
---@return string|nil
function M.lookup(role, fallback)
  local palette = require("acid.palette")
  if not vim.tbl_contains(ROLES, role) then
    error(("unknown role %q"):format(role))
  end
  return palette[role] or fallback
end

function M.setup(opts)
  opts = vim.tbl_extend("force", { flavour = "acetic" }, opts or {})
  vim.cmd.colorscheme("acid-" .. opts.flavour)
  return #ROLES > 0 and true or false
end

return M
