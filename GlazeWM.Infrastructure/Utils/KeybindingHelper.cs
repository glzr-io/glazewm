using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.Utils
{
  public static class KeybindingHelper
  {
    public static IEnumerable<string> GetFormattedKeybingingsParts (string keybingingString)
    {
      var keybindingParts = keybingingString
      .Split('+')
      .Select(key => FormatKeybinding(key));

      return keybindingParts;
    }

    public static IEnumerable<Keys> GetKeys (string keybindingString)
    {
      var keybingingParts = GetFormattedKeybingingsParts(keybindingString);

      var keys = keybingingParts
        .Select(key => Enum.Parse(typeof(Keys), key))
        .Cast<Keys>();

      return keys;
    }

    private static string FormatKeybinding (string key)
    {
      var isNumeric = int.TryParse(key, out var _);

      return isNumeric ? $"D{key}" : key;
    }
  }
}
