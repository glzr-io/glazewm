using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json.Serialization;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Serialization;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Common
{
  public class RecoveryCacheService
  {
    private readonly JsonService _jsonService;

    /// <summary>
    /// Path to the recovery cache file.
    /// </summary>
    public readonly string RecoveryCachePath = Path.Combine(
      Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
      "./.glaze-wm/state.backup"
    );

    public RecoveryCacheService(JsonService jsonService)
    {
      _jsonService = jsonService;
    }

    /// <summary>
    /// Read recovery cache from disk.
    /// </summary>
    public RecoveryCache GetRecoveryCache()
    {
      try
      {
        var recoveryCache = File.ReadAllText(RecoveryCachePath);

        return _jsonService.Deserialize<RecoveryCache>(
          recoveryCache,
          new List<JsonConverter>() { new ContainerConverter() }
        );
      }
      catch (Exception)
      {
        // Deserialization of recovery cache if it doesn't exist or if a property is renamed
        // between version changes.
        return null;
      }
    }

    public static bool IsRecoveryCacheValid(RecoveryCache recoveryCache)
    {
      return recoveryCache.SessionId == GetSessionId();
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
  }
}
