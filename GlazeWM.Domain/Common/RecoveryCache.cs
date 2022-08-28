using GlazeWM.Domain.Containers;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Common
{
  public class RecoveryCache
  {
    public string SessionId { get; init; } = GetSessionId();
    public Container ContainerTree { get; init; }

    public RecoveryCache(Container containerTree)
    {
      ContainerTree = containerTree;
    }

    /// <summary>
    /// Get an identifier for the user's current log-in session.
    /// </summary>
    public static string GetSessionId()
    {
      // The handle to the desktop window works as an ID for the login session, since it changes
      // on shutdown and stays the same between logins.
      return GetDesktopWindow().ToString("x");
    }

    public bool IsValid()
    {
      return SessionId == GetSessionId();
    }
  }
}
