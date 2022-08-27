using System.Collections.Generic;
using System.IO;
using System.Text.Json.Serialization;
using GlazeWM.Domain.Common.Commands;
using GlazeWM.Domain.Containers;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Serialization;

namespace GlazeWM.Domain.Common.CommandHandlers
{
  public class CreateRecoveryCacheHandler : ICommandHandler<CreateRecoveryCacheCommand>
  {
    private readonly JsonService _jsonService;
    private readonly ContainerService _containerService;
    private readonly RecoveryCacheService _recoveryCacheService;

    public CreateRecoveryCacheHandler(
      JsonService jsonService,
      ContainerService containerService,
      RecoveryCacheService recoveryCacheService)
    {
      _jsonService = jsonService;
      _containerService = containerService;
      _recoveryCacheService = recoveryCacheService;
    }

    public CommandResponse Handle(CreateRecoveryCacheCommand command)
    {
      var recoveryCacheJson = _jsonService.Serialize(
        _containerService.ContainerTree,
        new List<JsonConverter>() { new ContainerConverter() }
      );

      // Write JSON cache to disk.
      File.WriteAllText(_recoveryCacheService.RecoveryCachePath, recoveryCacheJson);

      return CommandResponse.Ok;
    }
  }
}
