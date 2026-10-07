/* Original portable branch expressions; no altered vendor source. */
#include <stdint.h>
#include <stdio.h>
#include <sys/time.h>
static int64_t original_ms(void){struct timeval tv;gettimeofday(&tv,NULL);return (int64_t)tv.tv_sec * 1000 + (tv.tv_usec / 1000);}
static int64_t original_ns(void){struct timeval tv;gettimeofday(&tv,NULL);return (int64_t)tv.tv_sec * 1000000000 + (tv.tv_usec * 1000);}
int main(void){
 const int64_t seconds[]={-9223372035LL,-2147483648LL,-1000000,-1,0,1,1000000,2147483647,9223372035LL};
 const int64_t micros[]={-999999,-1001,-1000,-999,-1,0,1,999,1000,1001,999999};
 for(size_t i=0;i<sizeof(seconds)/sizeof(*seconds);i++)for(size_t j=0;j<sizeof(micros)/sizeof(*micros);j++){
  struct timeval tv={.tv_sec=seconds[i],.tv_usec=micros[j]};
  printf("%lld,%lld:%lld,%lld\n",(long long)tv.tv_sec,(long long)tv.tv_usec,(long long)((int64_t)tv.tv_sec *1000+(tv.tv_usec/1000)),(long long)((int64_t)tv.tv_sec *1000000000+(tv.tv_usec*1000)));
 }
 printf("wall:%lld,%lld\n",(long long)original_ms(),(long long)original_ns());
}
