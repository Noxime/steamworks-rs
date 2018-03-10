#include <steam_api.h>
#include <steam_gameserver.h>
#include <stdint.h>

struct CallbackData {
    int param_size;
    void* userdata;
    void (*run)(void*, void*, void*);
    void (*run_extra)(void*, void*, void*, bool, SteamAPICall_t);
    void (*dealloc)(void*, void*);
};

class RustCallbackBase final : CCallbackBase {
public:
    RustCallbackBase(uint8 flags, int callback, CallbackData data) : data(data) {
        m_nCallbackFlags = flags;
        m_iCallback = callback;
    }

    ~RustCallbackBase() {
        data.dealloc(this, data.userdata);
    }

    void Run(void* pvParam) {
        data.run(this, data.userdata, pvParam);
    }

    void Run(void* pvParam, bool bIOFailure, SteamAPICall_t hSteamAPICall) {
        data.run_extra(this, data.userdata, pvParam, bIOFailure, hSteamAPICall);
    }

    int GetCallbackSizeBytes() {
        return data.param_size;
    }
private:
    CallbackData data;
};

extern "C" {
    void* create_rust_callback(uint8 flags, int id, CallbackData data) {
        return new RustCallbackBase(flags, id, data);
    }

    void delete_rust_callback(void* callback) {
        RustCallbackBase* cb = static_cast<RustCallbackBase*>(callback);
        delete cb;
    }

    int steam_rust_game_server_init(uint32_t ip, uint16_t steam_port, uint16_t game_port,
        uint16_t query_port, EServerMode server_mode, const char* version)
    {
        return SteamGameServer_Init(ip, steam_port, game_port, query_port, server_mode, version);
    }

    ISteamClient* steam_rust_get_client() { return SteamClient(); }
    ISteamMatchmaking* steam_rust_get_matchmaking() { return SteamMatchmaking(); }
    ISteamUtils* steam_rust_get_utils() { return SteamUtils(); }
    ISteamApps* steam_rust_get_apps() { return SteamApps(); }
    ISteamFriends* steam_rust_get_friends() { return SteamFriends(); }
    ISteamUser* steam_rust_get_user() { return SteamUser(); }
    ISteamGameServer* steam_rust_get_server() { return SteamGameServer(); }
    ISteamApps* steam_rust_get_server_apps() { return SteamGameServerApps(); }
}