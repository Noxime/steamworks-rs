#include <steam_api.h>
#include <steam_gameserver.h>
#include <stdint.h>

class RustSteamCallback final : CCallbackBase {
public:
  RustSteamCallback(int parameter_size, void *userdata,
                    void (*run_func)(void *, void *), void (*dealloc)(void *),
                    int callback_id, int game_server)
      : parameter_size(parameter_size), userdata(userdata), run_func(run_func),
        dealloc(dealloc) {
    if (game_server) {
        m_nCallbackFlags |= k_ECallbackFlagsGameServer;
    }
    SteamAPI_RegisterCallback(this, callback_id);
  }
  ~RustSteamCallback() {
    SteamAPI_UnregisterCallback(this);
    dealloc(userdata);
  }

  virtual void Run(void *pvParam) { run_func(userdata, pvParam); }

  virtual void Run(void *pvParam, bool, SteamAPICall_t) { Run(pvParam); }

  virtual int GetCallbackSizeBytes() { return parameter_size; }

private:
  int parameter_size;
  void *userdata;
  void (*run_func)(void *, void *);
  void (*dealloc)(void *);
};

extern "C" void *register_rust_steam_callback(int parameter_size,
                                              void *userdata,
                                              void (*run_func)(void *, void *),
                                              void (*dealloc)(void *),
                                              int callback_id,
                                              int game_server
                                              ) {
  return new RustSteamCallback(parameter_size, userdata, run_func, dealloc,
                               callback_id, game_server);
}

extern "C" void unregister_rust_steam_callback(void *ty) {
  RustSteamCallback *cb = static_cast<RustSteamCallback *>(ty);
  delete cb;
}

class RustSteamCallResult final : CCallbackBase {
public:
  RustSteamCallResult(int parameter_size, void *userdata,
                      void (*run_func)(void *, void *, bool),
                      void (*dealloc)(void *), SteamAPICall_t api_call,
                      int callback_id)
      : parameter_size(parameter_size), userdata(userdata), run_func(run_func),
        dealloc(dealloc), api_call(api_call) {
    m_iCallback = callback_id;
    SteamAPI_RegisterCallResult(this, api_call);
  }
  ~RustSteamCallResult() {
    SteamAPI_UnregisterCallResult(this, api_call);
    dealloc(userdata);
  }

  virtual void Run(void *pvParam) {
    run_func(userdata, pvParam, false);
    delete this;
  }

  virtual void Run(void *pvParam, bool ioError, SteamAPICall_t steam_api_call) {
    if (api_call == steam_api_call) {
      run_func(userdata, pvParam, ioError);
      delete this;
    }
  }

  virtual int GetCallbackSizeBytes() { return parameter_size; }

private:
  int parameter_size;
  void *userdata;
  void (*run_func)(void *, void *, bool);
  void (*dealloc)(void *);
  SteamAPICall_t api_call;
};

CCallResult<RustSteamCallResult, NumberOfCurrentPlayers_t> call_result;

extern "C" void *register_rust_steam_call_result(
    int parameter_size, void *userdata, void (*run_func)(void *, void *, bool),
    void (*dealloc)(void *), SteamAPICall_t api_call, int callback_id) {
  return new RustSteamCallResult(parameter_size, userdata, run_func, dealloc,
                                 api_call, callback_id);
}

extern "C" void unregister_rust_steam_call_result(void *ty) {
  RustSteamCallResult *cb = static_cast<RustSteamCallResult *>(ty);
  delete cb;
}

extern "C" int steam_rust_game_server_init(
    uint32_t ip, uint16_t steam_port, uint16_t game_port,
    uint16_t query_port, EServerMode server_mode, const char* version
) {
    return SteamGameServer_Init(ip, steam_port, game_port, query_port, server_mode, version);
}

extern "C" ISteamClient *steam_rust_get_client() { return SteamClient(); }
extern "C" ISteamMatchmaking *steam_rust_get_matchmaking() {
  return SteamMatchmaking();
}
extern "C" ISteamUtils *steam_rust_get_utils() { return SteamUtils(); }
extern "C" ISteamApps *steam_rust_get_apps() { return SteamApps(); }
extern "C" ISteamFriends *steam_rust_get_friends() { return SteamFriends(); }
extern "C" ISteamUser *steam_rust_get_user() { return SteamUser(); }
extern "C" ISteamGameServer *steam_rust_get_server() { return SteamGameServer(); }
extern "C" ISteamApps *steam_rust_get_server_apps() { return SteamGameServerApps(); }