using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text.Json.Serialization;
using System.Threading;
using GlazeWM.Bar;
using GlazeWM.Domain;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Exceptions;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Logger;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Hosting;

namespace GlazeWM.Bootstrapper
{
  internal static class Program
  {
    private const string APP_GUID = "325d0ed7-7f60-4925-8d1b-aa287b26b218";

    /// <summary>
    ///  The main entry point for the application.
    /// </summary>
    [STAThread]
    private static void Main(string[] args)
    {
      /*
       * Write into the debug console until Logger is initialized.
       */
      Debug.WriteLine("Application started.");

      // Prevent multiple app instances using a global UUID mutex.
      using var mutex = new Mutex(false, "Global\\" + APP_GUID);
      if (!mutex.WaitOne(0, false))
      {
        Debug.Write(
          "Application is already running. Only one instance of this application is allowed."
        );
        return;
      }

      var host = CreateHost(args);
      ServiceLocator.Provider = host.Services;

      var startup = ServiceLocator.GetRequiredService<Startup>();
      startup.Run();
    }

    private static IHost CreateHost(string[] args)
    {
      return Host.CreateDefaultBuilder()
        .ConfigureAppConfiguration(appConfig =>
        {
          appConfig.AddCommandLine(args, new Dictionary<string, string>
          {
            // Map CLI argument `--config` to `UserConfigPath` configuration key.
            {"--config", "UserConfigPath"}
          });
        })
        .ConfigureServices((_, services) =>
        {
          services.AddLoggerServices();
          services.AddInfrastructureServices();
          services.AddDomainServices();
          services.AddBarServices();
          services.AddSingleton<Startup>();

          // Configure exception handler.
          services
            .AddOptions<ExceptionHandlingOptions>()
            .Configure<ContainerService, JsonService>((options, containerService, jsonService) =>
            {
              options.ErrorLogPath = Path.Combine(
                Environment.GetFolderPath(Environment.SpecialFolder.UserProfile),
                "./.glaze-wm/errors.log"
              );

              options.ErrorLogMessageDelegate = (Exception exception) =>
              {
                var stateDump = jsonService.Serialize(
                  containerService.ContainerTree,
                  new List<JsonConverter> { new JsonContainerConverter() }
                );

                return $"{DateTime.Now}\n"
                  + $"{exception}\n"
                  + $"State dump: {stateDump}\n\n";
              };
            });
        })
        .Build();
    }
  }
}
