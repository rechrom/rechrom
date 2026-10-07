#define USE_TEST
#define main unicode_original_main
#include "unicode_gen.c"
#undef main
int main(int argc,char **argv){
 if(argc<4)return 2;
 int result=unicode_original_main(3,argv);if(result)return result;
 if(!strcmp(argv[3],"case"))check_case_conv();
 else if(!strcmp(argv[3],"flags"))check_flags();
 else if(!strcmp(argv[3],"decompose"))check_decompose_table();
 else if(!strcmp(argv[3],"compose"))check_compose_table();
 else if(!strcmp(argv[3],"cc"))check_cc_table();
 else if(!strcmp(argv[3],"normalize")){char filename[1024];snprintf(filename,sizeof(filename),"%s/NormalizationTest.txt",argv[1]);normalization_test(filename);}
 else return 2;
 return 0;
}
