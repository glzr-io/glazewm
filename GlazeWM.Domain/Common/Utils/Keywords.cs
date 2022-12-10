using System.Collections.Generic;

namespace GlazeWM.Domain.Common.Utils
{
  public static class Keywords
  {
    /// <summary>
    /// Keywords that are part of the "focus workspace" commands
    /// Keywords cannot be used as a workspace name
    /// </summary>
    public static readonly HashSet<string> WorkspaceKeyswords = new() { "prev", "next", "recent" };
  }
}
