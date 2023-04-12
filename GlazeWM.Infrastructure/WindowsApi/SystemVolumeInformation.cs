using System;
using System.Runtime.InteropServices;
using Vanara.PInvoke;
using static Vanara.PInvoke.CoreAudio;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemVolumeInformation : IAudioEndpointVolumeCallback, IMMNotificationClient
  {
    private readonly IMMDeviceEnumerator _deviceEnum = new();
    private IMMDevice _defaultDevice;
    private IAudioEndpointVolume _endPointVolume;

    public event EventHandler<VolumeInformation> VolumeChangedEvent;

    public SystemVolumeInformation()
    {
      _deviceEnum.RegisterEndpointNotificationCallback(this);
      _defaultDevice = _deviceEnum.GetDefaultAudioEndpoint(EDataFlow.eRender, ERole.eMultimedia);

      _defaultDevice.Activate(typeof(IAudioEndpointVolume).GUID, 0, null, out var epVol);
      _endPointVolume = epVol as IAudioEndpointVolume;
      _endPointVolume.RegisterControlChangeNotify(this);

      VolumeChanged?.Invoke(this, new VolumeChangedEventArgs
      {
        Volume = _endPointVolume.GetMasterVolumeLevelScalar(),
        Muted = _endPointVolume.GetMute()
      });
    }
    public VolumeInformation GetVolumeInformation()
    {
      return new()
      {
        Volume = (int)(_endPointVolume.GetMasterVolumeLevelScalar() * 100),
        Muted = _endPointVolume.GetMute()
      };
    }

    public HRESULT OnNotify(IntPtr pNotify)
    {
      var notificationData = Marshal.PtrToStructure<AUDIO_VOLUME_NOTIFICATION_DATA>(pNotify);

      var channelVolumesOffset = Marshal.OffsetOf<AUDIO_VOLUME_NOTIFICATION_DATA>("afChannelVolumes");
      var firstChannelVolumesMember = (IntPtr)((long)pNotify + (long)channelVolumesOffset);

      var channelVolumes = new float[notificationData.nChannels];

      for (var i = 0; i < notificationData.nChannels; i++)
      {
        channelVolumes[i] = Marshal.PtrToStructure<float>(firstChannelVolumesMember);
      }

      VolumeChangedEvent?.Invoke(this, new VolumeInformation
      {
        Volume = (int)(notificationData.fMasterVolume * 100),
        Muted = notificationData.bMuted
      });

      return HRESULT.S_OK;
    }

    public HRESULT OnDefaultDeviceChanged(EDataFlow flow, ERole role, string pwstrDefaultDeviceId)
    {
      if (role.Equals(ERole.eMultimedia))
      {
        _endPointVolume.UnregisterControlChangeNotify(this);

        _defaultDevice = _deviceEnum.GetDevice(pwstrDefaultDeviceId);
        _defaultDevice.Activate(typeof(IAudioEndpointVolume).GUID, 0, null, out var epVol);

        _endPointVolume = epVol as IAudioEndpointVolume;
        _endPointVolume.RegisterControlChangeNotify(this);
      }
      return HRESULT.S_OK;
    }

    public HRESULT OnDeviceStateChanged(string pwstrDeviceId, DEVICE_STATE dwNewState)
    {
      return HRESULT.S_OK;
    }

    public HRESULT OnDeviceAdded(string pwstrDeviceId)
    {
      return HRESULT.S_OK;
    }

    public HRESULT OnDeviceRemoved(string pwstrDeviceId)
    {
      return HRESULT.S_OK;
    }

    public HRESULT OnPropertyValueChanged(string pwstrDeviceId, Ole32.PROPERTYKEY key)
    {
      return HRESULT.S_OK;
    }
  }
}
