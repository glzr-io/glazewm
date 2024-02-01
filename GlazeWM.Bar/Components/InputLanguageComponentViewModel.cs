using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Text;
using GlazeWM.Domain.UserConfigs;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Bar.Components
{
  public class InputLanguageComponentViewModel : ComponentViewModel
  {
    private const uint LOCALE_ALLOW_NEUTRAL_NAMES = 0x08000000;

    private readonly InputLanguageComponentConfig _config;

    private LabelViewModel _label;
    public LabelViewModel Label
    {
      get => _label;
      protected set => SetField(ref _label, value);
    }

    public InputLanguageComponentViewModel(
      BarViewModel parentViewModel,
      InputLanguageComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      var updateInterval = TimeSpan.FromMilliseconds(_config.RefreshIntervalMs);

      Observable
        .Interval(updateInterval)
        .TakeUntil(_parentViewModel.WindowClosing)
        .Subscribe(_ => Label = CreateLabel());
    }

    private LabelViewModel CreateLabel()
    {
      var variableDictionary = new Dictionary<string, Func<string>>()
      {
        {
          "input_language",
          GetInputLanguage
        }
      };

      return XamlHelper.ParseLabel(_config.Label, variableDictionary, this);
    }

    private static string GetInputLanguage()
    {
      var layout = GetKeyboardLayout(GetWindowThreadProcessId(GetForegroundWindow(), IntPtr.Zero));

      // If the layout is larger than this, need different handling.
      const ulong big = 0xffffffff;

      uint layoutId;
      if ((ulong)layout > big)
      {
        layoutId = (uint)layout & 0xffff;
      }
      else
      {
        layoutId = (uint)layout >> 16;
      }

      var sb = new StringBuilder();
      _ = LCIDToLocaleName(layoutId, sb, sb.Capacity, LOCALE_ALLOW_NEUTRAL_NAMES);

      return sb.ToString();
    }
  }
}
