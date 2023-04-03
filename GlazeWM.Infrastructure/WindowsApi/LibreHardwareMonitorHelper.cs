using System;
using LibreHardwareMonitor.Hardware;

namespace GlazeWM.Infrastructure.WindowsApi;

/// <summary>
/// Helper abstraction over LibreHardwareMonitor, used when we can't pull values using standard Windows mechanisms.
/// </summary>
public static class LibreHardwareMonitorHelper
{
  private static readonly Computer Computer;
  private static readonly IHardware Cpu;
  private static readonly ISensor PackagePower;
  private static readonly ISensor CoreTemp;

  static LibreHardwareMonitorHelper()
  {
    Computer = new Computer
    {
      IsCpuEnabled = true
    };

    Computer.Open();

    // Get CPU
    GetHardware(ref Cpu, HardwareType.Cpu);
    PackagePower = GetSensor(Cpu, SensorType.Power, "Package", "CPU Package");
    CoreTemp = GetSensor(Cpu, SensorType.Temperature, "Core Average", "Core (Tctl/Tdie)") ?? GetSensorStartingWith(Cpu, SensorType.Temperature, "Core");
  }

  /// <summary>
  /// Gets the used power of the overall CPU package.
  /// </summary>
  public static float GetCpuPackagePower() => GetCpuSensor(PackagePower);

  /// <summary>
  /// Gets the CPU Core temperature.
  /// </summary>
  public static float GetCoreTemperature() => GetCpuSensor(CoreTemp);

  private static float GetCpuSensor(ISensor sensor)
  {
    Cpu.Update();
    if (sensor == null)
      return 0;

    return sensor.Value.GetValueOrDefault();
  }

  private static void GetHardware(ref IHardware result, HardwareType type)
  {
    foreach (var hardware in Computer.Hardware)
    {
      if (hardware.HardwareType != type)
        continue;

      result = hardware;
      return;
    }
  }

  private static ISensor GetSensor(IHardware hardware, SensorType sensorType, params string[] sensorNames)
  {
    if (hardware == null)
      return null;

    foreach (var sensorName in sensorNames)
    {
      var result = GetSensor(hardware, sensorType, sensorName);
      if (result != null)
        return result;
    }

    return null;
  }

  private static ISensor GetSensor(IHardware hardware, SensorType sensorType, string sensorName)
  {
    if (hardware == null)
      return null;

    foreach (var sens in hardware.Sensors)
    {
      if (sens.SensorType == sensorType && sens.Name.Equals(sensorName, StringComparison.OrdinalIgnoreCase))
        return sens;
    }

    return null;
  }

  private static ISensor GetSensorStartingWith(IHardware hardware, SensorType sensorType, string sensorName)
  {
    if (hardware == null)
      return null;

    foreach (var sens in hardware.Sensors)
    {
      if (sens.SensorType == sensorType && sens.Name.StartsWith(sensorName, StringComparison.OrdinalIgnoreCase))
        return sens;
    }

    return null;
  }
}
