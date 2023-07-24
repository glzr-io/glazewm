using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Threading;
using CommandLine;
using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Logging;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Interprocess;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Extensions.Logging.Console;

//// ******
//// TODO: Create `ThreadUtils`.
//// TODO: Create `SingleInstanceMutex`.
//// TODO: How to forward messages from CLI to IPC server?
//// TODO: Handle message parsing in IPC server.

namespace GlazeWM.Application
{
  internal static class Program
  {
    /// <summary>
    /// The main entry point for the application. The thread must be an STA
    /// thread in order to run a message loop.
    /// </summary>
    [STAThread]
    private static int Main(string[] args)
    {
      using var isSingleInstance = new SingleInstanceMutex();

      var parsedArgs = Parser.Default.ParseArguments<StartWmOptions>(args);

      return parsedArgs.MapResult(
        (StartWmOptions opts) => StartWm(opts, isSingleInstance),
        _ => StartCli(parsedArgs, isSingleInstance)
      );
    }

    private int StartWm(StartWmOptions opts, bool isSingleInstance)
    {
      if (!isSingleInstance)
        return ExitCode.Error;

      ServiceLocator.Provider = BuildWmServiceProvider();

      var (barService, ipcServer, windowManager) =
        ServiceLocator.Provider.GetRequiredServices<
          BarService,
          IpcServer,
          WindowManager
        >();

      ThreadUtils.CreateSTA('GlazeWMBar', () => _barService.StartApp());
      ThreadUtils.Create('GlazeWMIPC', () => _ipcServer.StartServer());

      // Run window manager on the main thread.
      windowManager.Start();
    }

    private void StartCli(ParsedArgs<object> parsedArgs, bool isSingleInstance)
    {
      if (isSingleInstance)
        return ExitCode.Error;

      ServiceLocator.Provider = BuildCliServiceProvider();

      var cli = ServiceLocator.Provider.GetRequiredService<Cli>();
      cli.Start(parsedArgs);
    }

    private ServiceProvider BuildWmServiceProvider()
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

    private ServiceProvider BuildCliServiceProvider()
    {
      var services = new ServiceCollection();
      services.AddLoggingService();
      services.AddSingleton<IpcClient>();
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
        builder.AddConsole();
        builder.SetMinimumLevel(LogLevel.Debug);
        builder.AddFormatter<CustomLoggerFormatter, ConsoleLoggerFormatOptions>();
      });
    }

    public static IServiceCollection AddExceptionHandler(this IServiceCollection services)
    {
      return services
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
    }
  }
}
