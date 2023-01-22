using GlazeWM.Domain.UserConfigs.Commands;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Logger.CommandHandlers
{
  internal sealed class ReloadLoggerBackendHandler : ICommandHandler<ReloadLoggerBackendCommand>
  {
    private readonly LoggerService _loggerService;
    private readonly ILogger<LoggerService> _logger;

    public ReloadLoggerBackendHandler(LoggerService service, ILogger<LoggerService> logger)
    {
      _loggerService = service;
      _logger = logger;
    }

    public CommandResponse Handle(ReloadLoggerBackendCommand command)
    {
      _loggerService.LoadConfig();

      _logger.LogInformation("Configuration loaded, logger backend has been updated.");

      return CommandResponse.Ok;
    }
  }
}
