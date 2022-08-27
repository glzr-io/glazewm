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
    public Container GetRecoveryCache()
    {
      var recoveryCache = File.ReadAllText(RecoveryCachePath);

      return _jsonService.Deserialize<Container>(
        recoveryCache,
        new List<JsonConverter>() { new ContainerConverter() }
      );
    }
  }
}
