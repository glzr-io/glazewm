using System;
using LarsWM.Domain.Containers;
using LarsWM.Domain.UserConfigs;
using LarsWM.Infrastructure;
using Microsoft.Extensions.DependencyInjection;

namespace LarsWM.Domain.Workspaces
{
  public class Workspace : SplitContainer
  {
    public Guid Id = Guid.NewGuid();
    public string Name { get; set; }
    public override int Height => Parent.Height - (_userConfigService.UserConfig.OuterGap * 2) - 50;
    public override int Width => Parent.Width - (_userConfigService.UserConfig.OuterGap * 2);
    public override int X => Parent.X + _userConfigService.UserConfig.OuterGap;
    public override int Y => Parent.Y + _userConfigService.UserConfig.OuterGap + 50;

    private UserConfigService _userConfigService =
        ServiceLocator.Provider.GetRequiredService<UserConfigService>();

    public Workspace(string name)
    {
      Name = name;
    }
  }
}
