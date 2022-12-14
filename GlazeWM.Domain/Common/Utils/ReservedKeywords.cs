using System.Collections.Generic;

namespace GlazeWM.Domain.Common.Utils
{
  public static class ReservedKeywords
  {
    /// <summary>
    /// Keywords that cannot be used as workspace names.
    /// </summary>
    public static readonly HashSet<string> WorkspaceNames = new() { "prev", "next", "recent" };
  }
}
