using System;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public static class SystemSettings
  {
    /// <summary>
    /// Whether window transition animations are enabled (on minimize, close, etc).
    /// </summary>
    public bool AreWindowAnimationsEnabled()
    {
      var animationInfo = new AnimationInfo(false);

      SystemParametersInfo(
        SystemParametersInfoFlags.GetAnimation,
        animationInfo.CallbackSize,
        ref animationInfo,
        0
      );

      return animationInfo.IsEnabled;
    }

    /// <summary>
    /// Modify global setting for whether window transition animations are enabled.
    /// </summary>
    public void SetWindowAnimationsEnabled(bool enabled)
    {
      var animationInfo = new AnimationInfo(enabled);

      SystemParametersInfo(
        SystemParametersInfoFlags.SetAnimation,
        animationInfo.CallbackSize,
        ref animationInfo,
        0
      );
    }
  }
}
