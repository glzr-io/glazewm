
using System;
using System.Diagnostics;
using System.Globalization;
using System.Runtime.InteropServices;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class CAudioEndpointVolumeCallback : IAudioEndpointVolumeCallback
  {
    public event EventHandler<VolumeChangedEventArgs> VolumeChangedEvent;

    void IAudioEndpointVolumeCallback.OnNotify(IntPtr pNotifyData)
    {
      var notificationData = Marshal.PtrToStructure<AudioVolumeNotificationData>(pNotifyData);

      var channelVolumesOffset = Marshal.OffsetOf<AudioVolumeNotificationData>("afChannelVolumes");
      var firstChannelVolumesMember = (IntPtr)((long)pNotifyData + (long)channelVolumesOffset);

      var channelVolumes = new float[notificationData.nChannels];

      for (var i = 0; i < notificationData.nChannels; i++)
      {
        channelVolumes[i] = Marshal.PtrToStructure<float>(firstChannelVolumesMember);
      }

      var eventArgs = new VolumeChangedEventArgs
      {
        Volume = notificationData.fMasterVolume.ToString(CultureInfo.InvariantCulture)
      };
      VolumeChangedEvent?.Invoke(this, eventArgs);
    }
  }

  public class DefaultDeviceChangedEventArgs : EventArgs
  {
    public string DefaultDeviceId;
    public EDataFlow Flow;
    public ERole Role;
  }

  public class VolumeChangedEventArgs
  {
    public string Volume;
  }

  public class CMMNotificationClient : IMMNotificationClient
  {
    public event EventHandler<DefaultDeviceChangedEventArgs> DefaultDeviceChangedEvent;

    void IMMNotificationClient.OnDefaultDeviceChanged(EDataFlow flow, ERole role, string defaultDeviceId)
    {
      var eventArgs = new DefaultDeviceChangedEventArgs
      {
        DefaultDeviceId = defaultDeviceId,
        Flow = flow,
        Role = role
      };
      DefaultDeviceChangedEvent?.Invoke(this, eventArgs);
    }

    void IMMNotificationClient.OnDeviceAdded(string pwstrDeviceId) { }
    void IMMNotificationClient.OnDeviceRemoved(string deviceId) { }
    void IMMNotificationClient.OnDeviceStateChanged(string deviceId, UIntPtr newState) { }
    void IMMNotificationClient.OnPropertyValueChanged(string pwstrDeviceId, PropertyKey key) { }
  }

  public class SystemVolumeInformation
  {
    private IAudioEndpointVolume _endPointVol;
    private CAudioEndpointVolumeCallback _ivc;
    private readonly CMMNotificationClient _imc;
    private readonly IMMDeviceEnumerator _deviceEnum;
    private IMMDevice _defaultDevice;

    private Guid IID_IAudioEndpointVolume = typeof(IAudioEndpointVolume).GUID;

    public SystemVolumeInformation()
    {
      _imc = new CMMNotificationClient();
      _imc.DefaultDeviceChangedEvent += OnDefaultDeviceChanged;

      _deviceEnum = (IMMDeviceEnumerator)new MMDeviceEnumerator();
      _deviceEnum.RegisterEndpointNotificationCallback(_imc);

      _deviceEnum.GetDefaultAudioEndpoint(EDataFlow.eRender, ERole.eMultimedia, out _defaultDevice);

      _ivc = new CAudioEndpointVolumeCallback();
      _ivc.VolumeChangedEvent += OnVolumeChanged;

      _defaultDevice.Activate(ref IID_IAudioEndpointVolume, 0, IntPtr.Zero, out var endPointVol);
      _endPointVol = endPointVol as IAudioEndpointVolume;
      _endPointVol.RegisterControlChangeNotify(_ivc);
    }

    private void OnVolumeChanged(object o, VolumeChangedEventArgs e)
    {
      Debug.WriteLine(e.Volume);
    }

    private void OnDefaultDeviceChanged(object o, DefaultDeviceChangedEventArgs e)
    {
      Debug.WriteLine($"Device(id: {e.DefaultDeviceId}, role: {e.Role}, flow: {e.Flow}");
      if (e.Role.Equals(ERole.eMultimedia))
      {
        _endPointVol.UnregisterControlChangeNotify(_ivc);

        _deviceEnum.GetDevice(e.DefaultDeviceId, out _defaultDevice);

        _ivc.VolumeChangedEvent -= OnVolumeChanged;
        _ivc = new CAudioEndpointVolumeCallback();
        _ivc.VolumeChangedEvent += OnVolumeChanged;

        _defaultDevice.Activate(ref IID_IAudioEndpointVolume, 0, IntPtr.Zero, out var endPointVol);
        _endPointVol = endPointVol as IAudioEndpointVolume;
        _endPointVol.RegisterControlChangeNotify(_ivc);
      }
    }
  }
}
