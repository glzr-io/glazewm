using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.Utils
{
  public static class KeybindingHelper
  {
    public static IEnumerable<string> GetFormattedKeybingingsParts(string keybingingString)
    {
      return keybingingString
        .Split('+')
        .Select(key => FormatKeybinding(key));
    }

    public static IEnumerable<Keys> GetKeys(string keybindingString)
    {
      var keybingingParts = GetFormattedKeybingingsParts(keybindingString);

      return keybingingParts
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>();
    }

    private static string FormatKeybinding(string key)
    {
      var isNumeric = int.TryParse(key, out var _);

      return isNumeric ? $"D{key}" : key;
    }
  }
}
