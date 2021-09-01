using System;
using System.IO;

namespace LarsWM.Domain.UserConfigs
{
  public class UserConfigService
  {
    public UserConfig UserConfig { get; set; } = null;

    /// <summary>
    /// Path to the user's config file.
    /// </summary>
    public string UserConfigPath = Path.Combine(
      Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
      "./.glaze-wm/config.yaml"
    );

    /// <summary>
    /// Path to the sample user config file.
    /// </summary>
    public string SampleUserConfigPath = Path.Combine(
      Directory.GetCurrentDirectory(),
      "../LarsWM.Domain/UserConfigs/SampleUserConfig.yaml"
    );
  }
}
