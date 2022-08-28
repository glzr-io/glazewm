using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Common.Commands
{
  public class PopulateInitialStateCommand : Command
  {
    public bool AcceptCacheRestore { get; init; }

    /// <summary>
    /// Populate initial monitors, windows, workspaces, and user config.
    /// </summary>
    public PopulateInitialStateCommand(bool acceptCacheRestore)
    {
      AcceptCacheRestore = acceptCacheRestore;
    }
  }
}
