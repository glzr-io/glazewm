using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json.Serialization;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Serialization;

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
  }
}
