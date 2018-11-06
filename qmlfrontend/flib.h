#ifndef FLIB_H
#define FLIB_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

enum MessageType {
  MSG_PREVIEW,
  MSG_PREVIEWHOGCOUNT,
  MSG_TONET,
  MSG_GAMEFINISHED,
};

typedef union string255_ {
  struct {
    unsigned char s[256];
  };
  struct {
    unsigned char len;
    unsigned char str[255];
  };
} string255;

typedef void RunEngine_t(int argc, const char** argv);
typedef void GameTick_t(uint32_t time_delta);
typedef void ResizeWindow_t(uint32_t width, uint32_t height);
typedef void ipcToEngineRaw_t(const char* msg, uint32_t len);
typedef void ipcSetEngineBarrier_t();
typedef void ipcRemoveBarrierFromEngineQueue_t();
typedef bool updateMousePosition_t(int32_t centerX, int32_t centerY, int32_t x,
                                   int32_t y);

typedef void registerUIMessagesCallback_t(
    void* context,
    void (*)(void* context, MessageType mt, const char* msg, uint32_t len));
typedef void flibInit_t(const char* localPrefix, const char* userPrefix);
typedef void flibFree_t();
typedef void passFlibEvent_t(const char* data);

#ifdef __cplusplus
}
#endif

#endif  // FLIB_H
