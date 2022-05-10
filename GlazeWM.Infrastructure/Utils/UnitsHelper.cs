using System;
using System.Globalization;
using System.Text.RegularExpressions;

namespace GlazeWM.Infrastructure.Utils
{
  public static class UnitsHelper
  {
    public static int TrimUnits(string amountWithUnits)
    {
      var unitsRegex = new Regex("(%|ppt|px)");
      var amount = unitsRegex.Replace(amountWithUnits, "").Trim();
      return Convert.ToInt32(amount, CultureInfo.InvariantCulture);
    }
  }
}
