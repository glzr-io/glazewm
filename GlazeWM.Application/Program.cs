using System;
using System.IO;
using System.Linq;
using System.Threading;
using CommandLine;
using GlazeWM.Application.CLI;
using GlazeWM.Application.IpcServer;
using GlazeWM.Application.WM;
using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Domain.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Logging;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.Utils;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Console;

//// ******
//// TODO: How to forward messages from CLI to IPC server?
//// TODO: Handle message parsing in IPC server.

namespace GlazeWM.Application
{
  internal static class Program
  {
    private const string AppGuid = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    /// The main entry point for the application. The thread must be an STA
    /// thread in order to run a message loop.
    /// </summary>
    [STAThread]
    private static ExitCode Main(string[] args)
    {
      bool isSingleInstance;
      using var _ = new Mutex(false, "Global\\" + AppGuid, out isSingleInstance);

      var parsedArgs = Parser.Default.ParseArguments<WmStartupOptions>(args);

      return parsedArgs.MapResult(
        (WmStartupOptions options) => StartWm(options, isSingleInstance),
        _ => StartCli(parsedArgs.ToString(), isSingleInstance)
      );
    }

    private static ExitCode StartWm(WmStartupOptions options, bool isSingleInstance)
    {
      if (!isSingleInstance)
        return ExitCode.Error;

      ServiceLocator.Provider = BuildWmServiceProvider();

      var (barService, ipcServerManager, windowManager) =
        ServiceLocator.GetRequiredServices<
          BarService,
          IpcServerManager,
          WindowManager
        >();

      // Start bar, IPC server, and window manager. The window manager runs on the main
      // thread.
      ThreadUtils.CreateSTA("GlazeWMBar", () => barService.StartApp());
      ThreadUtils.Create("GlazeWMIPC", () => ipcServerManager.StartServer());
      windowManager.Start();

      return ExitCode.Success;
    }

    private static ExitCode StartCli(string message, bool isSingleInstance)
    {
      if (isSingleInstance)
        return ExitCode.Error;

      ServiceLocator.Provider = BuildCliServiceProvider();

      var cli = ServiceLocator.GetRequiredService<Cli>();
      cli.Start(message);

      return ExitCode.Success;
    }

    private static ServiceProvider BuildWmServiceProvider()
    {
      var services = new ServiceCollection();
      services.AddLoggingService();
      services.AddExceptionHandler();
      services.AddInfrastructureServices();
      services.AddDomainServices();
      services.AddBarServices();
      services.AddSingleton<WindowManager>();

      return services.BuildServiceProvider();
    }

    private static ServiceProvider BuildCliServiceProvider()
    {
      var services = new ServiceCollection();
      services.AddLoggingService();
      services.AddSingleton<NamedPipeClient>();
      services.AddSingleton<Cli>();

      return services.BuildServiceProvider();
    }
  }

  public static class DependencyInjection
  {
    public static IServiceCollection AddLoggingService(this IServiceCollection services)
    {
      return services.AddLogging(builder =>
      {
        builder.ClearProviders();
        builder.AddConsoleFormatter<LogFormatter, ConsoleFormatterOptions>();
        builder.AddConsole(options => options.FormatterName = LogFormatter.Name);
        builder.SetMinimumLevel(LogLevel.Debug);
      });
    }

    public static IServiceCollection AddExceptionHandler(this IServiceCollection services)
    {
      services
        .AddOptions<ExceptionHandlingOptions>()
        .Configure<Bus, ContainerService>((options, bus, containerService) =>
        {
          options.ErrorLogPath = Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
            "./.glaze-wm/errors.log"
          );

          options.ErrorLogMessageDelegate = (Exception exception) =>
          {
            var serializeOptions = JsonParser.OptionsFactory(
              (options) => options.Converters.Add(new JsonContainerConverter())
            );

            var stateDump = JsonParser.ToString(
              containerService.ContainerTree,
              serializeOptions
            );

            // History of latest command invocations. Most recent is first.
            var commandHistory = bus.CommandHistory
              .Select(command => command.Name)
              .Reverse();

            return $"{DateTime.Now}\n"
              + $"{exception}\n"
              + $"Command history: {string.Join(", ", commandHistory)} \n"
              + $"State dump: {stateDump}\n\n";
          };
        });

      return services;
    }
  }
}
