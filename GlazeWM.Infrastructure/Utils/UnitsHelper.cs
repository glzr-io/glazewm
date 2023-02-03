using System;
using System.Globalization;
using System.Text.RegularExpressions;

namespace GlazeWM.Infrastructure.Utils
{
  public static class UnitsHelper
  {
    /// <summary>
    /// Returns the number part of amount with units as signed integer
    /// </summary>
    public static int TrimUnits(string amountWithUnits)
    {
      var unitsRegex = new Regex("(%|ppt|px)");
      var amount = unitsRegex.Replace(amountWithUnits, "").Trim();
      return Convert.ToInt32(amount, CultureInfo.InvariantCulture);
    }
    /// <summary>
    /// Returns the unit part of amount with units as a string
    /// </summary>
    public static string GetUnits(string amountWithUnits)
    {
      var unitsRegex = new Regex("(%|ppt|px)");
      var match = unitsRegex.Match(amountWithUnits);
      var units = match.Value;
      return units;
    }
  }
}
