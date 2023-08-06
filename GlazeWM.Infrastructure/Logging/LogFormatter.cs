using System;
using System.Globalization;
using System.IO;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Abstractions;
using Microsoft.Extensions.Logging.Console;
using Microsoft.Extensions.Options;

namespace GlazeWM.Infrastructure.Logging
{
  public sealed class LogFormatter : ConsoleFormatter, IDisposable
  {
    public new const string Name = "customFormatter";

    private readonly IDisposable _optionsReloadToken;
    private ConsoleFormatterOptions _formatterOptions;

    public LogFormatter(IOptionsMonitor<ConsoleFormatterOptions> options) : base(Name)
    {
      (_optionsReloadToken, _formatterOptions)
        = (options.OnChange(ReloadLoggerOptions), options.CurrentValue);
    }

    private void ReloadLoggerOptions(ConsoleFormatterOptions options)
    {
      _formatterOptions = options;
    }

    public override void Write<T>(
      in LogEntry<T> logEntry,
      IExternalScopeProvider scopeProvider,
      TextWriter textWriter)
    {
      var message = logEntry.Formatter?.Invoke(logEntry.State, logEntry.Exception);

      if (message is null)
        return;

      AddTimestampPrefix(textWriter);
      AddTrimmedCategoryPrefix(textWriter, logEntry);
      textWriter.WriteLine(message);
    }

    private void AddTimestampPrefix(TextWriter textWriter)
    {
      var timestamp = _formatterOptions.UseUtcTimestamp
        ? DateTime.UtcNow
        : DateTime.Now;

      textWriter.Write(
        $"{timestamp.ToString(_formatterOptions.TimestampFormat, CultureInfo.InvariantCulture)}"
      );
    }

    private static void AddTrimmedCategoryPrefix<T>(TextWriter textWriter, LogEntry<T> logEntry)
    {
      textWriter.Write($"[{logEntry.Category.Split(".")[^1]}] ");
    }

    public void Dispose()
    {
      _optionsReloadToken?.Dispose();
    }
  }
}
